//! CEO Builder backend logic.
//!
//! Contains the functions for building CEO entries, importing ceo_data.ccd,
//! and querying trait CEOs from the Assembly Kit data.

use anyhow::{anyhow, Result};

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::slice::from_ref;

use rpfm_lib::files::{Container, ContainerPath, db::DB, FileType, loc::Loc, pack::Pack, RFile, RFileDecoded, table::DecodedData};
use rpfm_lib::schema::*;

use rpfm_ipc::messages::CeoEntryData;

use rpfm_telemetry::*;

/// Build CEO entries (armour, career, traits, loc) from structured input data.
///
/// This is the core logic for the `BuildCeoEntries` command, extracted to keep
/// the background thread function manageable and to help the compiler optimize.
pub fn build_ceo_entries(
    pack: &mut Pack,
    schema: &Schema,
    entries: &[CeoEntryData],
) -> Result<Vec<ContainerPath>> {
        let mut added_paths: Vec<ContainerPath> = Vec::new();

        // ── helpers ────────────────────────────────────────────
        // Stable CRC32-based synthetic ID for auto_id fields.
        // These columns exist only in AKit XML, not in binary pack format.
        fn crc32_id(data: &[u8]) -> i64 {
            let mut crc: u32 = 0xFFFF_FFFF;
            for &b in data {
                crc ^= b as u32;
                for _ in 0..8 {
                    if crc & 1 != 0 { crc = (crc >> 1) ^ 0xEDB8_8320; }
                    else { crc >>= 1; }
                }
            }
            (1_500_000_000u32 + (!crc % 647_000_000u32)) as i64
        }

        fn auto_id(table: &str, row_key: &str) -> i64 {
            crc32_id(format!("{}|{}", table, row_key).as_bytes())
        }

        // Get-or-create a DB table file in the pack.
        // Returns the path so the caller can track it.
        fn get_or_create_db(
            pack: &mut Pack,
            schema: &Schema,
            table_name: &str,
            file_stem: &str,
        ) -> Result<String> {
            let path = format!("ceo_db/{}/{}", table_name, file_stem);
            let container_path = ContainerPath::File(path.clone());

            // If the file doesn't exist yet, create it.
            if pack.files_by_paths(from_ref(&container_path), false).is_empty() {
                let _full_table_name = format!("{}_tables", table_name.trim_end_matches("_tables"));
                let table_name_with_suffix = if table_name.ends_with("_tables") {
                    table_name.to_string()
                } else {
                    format!("{}_tables", table_name)
                };

                let def = schema.definitions_by_table_name(&table_name_with_suffix)
                    .and_then(|defs| defs.first())
                    .ok_or_else(|| anyhow!("No schema definition for {}", table_name_with_suffix))?;

                let patches = schema.patches_for_table(&table_name_with_suffix);
                let db = DB::new(def, patches, &table_name_with_suffix);
                let rfile = RFile::new_from_decoded(
                    &RFileDecoded::DB(db), 0, &path
                );
                pack.insert(rfile)?;
            }
            Ok(path)
        }

        // Insert a row into a DB table. Builds the row by matching field names
        // from the definition against the provided HashMap of name→value strings.
        fn insert_row(
            pack: &mut Pack,
            schema: &Schema,
            table_name: &str,
            file_stem: &str,
            values: &std::collections::HashMap<&str, String>,
        ) -> Result<ContainerPath> {
            let path = get_or_create_db(pack, schema, table_name, file_stem)?;
            let container_path = ContainerPath::File(path.clone());

            let _table_name_with_suffix = if table_name.ends_with("_tables") {
                table_name.to_string()
            } else {
                format!("{}_tables", table_name)
            };

            let mut files = pack.files_by_type_and_paths_mut(
                &[FileType::DB],
                from_ref(&container_path),
                true,
            );

            if let Some(file) = files.first_mut() {
                if let Ok(RFileDecoded::DB(db)) = file.decoded_mut() {
                    let fields = db.definition().fields_processed().to_vec();
                    let mut row: Vec<DecodedData> = Vec::new();

                    for field in &fields {
                        let fname = field.name();
                        let raw = values.get(fname).map(|s| s.as_str()).unwrap_or("");

                        let cell = match field.field_type() {
                            FieldType::Boolean =>
                                DecodedData::Boolean(raw == "true" || raw == "1"),
                            FieldType::I16 =>
                                DecodedData::I16(raw.parse().unwrap_or(0)),
                            FieldType::I32 =>
                                DecodedData::I32(raw.parse().unwrap_or(0)),
                            FieldType::I64 =>
                                DecodedData::I64(raw.parse().unwrap_or(0)),
                            FieldType::F32 =>
                                DecodedData::F32(raw.parse().unwrap_or(0.0)),
                            FieldType::F64 =>
                                DecodedData::F64(raw.parse().unwrap_or(0.0)),
                            FieldType::OptionalI16 =>
                                DecodedData::OptionalI16(raw.parse().unwrap_or(0)),
                            FieldType::OptionalI32 =>
                                DecodedData::OptionalI32(raw.parse().unwrap_or(0)),
                            FieldType::OptionalI64 =>
                                DecodedData::OptionalI64(raw.parse().unwrap_or(0)),
                            FieldType::OptionalStringU8 =>
                                DecodedData::OptionalStringU8(raw.to_string()),
                            FieldType::OptionalStringU16 =>
                                DecodedData::OptionalStringU16(raw.to_string()),
                            FieldType::StringU16 =>
                                DecodedData::StringU16(raw.to_string()),
                            _ =>
                                DecodedData::StringU8(raw.to_string()),
                        };
                        row.push(cell);
                    }
                    db.data_mut().push(row);
                }
            }
            Ok(container_path)
        }

        // ── loc helper ─────────────────────────────────────────
        fn insert_loc_entries(
            pack: &mut Pack,
            loc_path: &str,
            entries: &[(&str, &str)], // (key, text)
        ) -> Result<ContainerPath> {
            let container_path = ContainerPath::File(loc_path.to_string());

            if pack.files_by_paths(from_ref(&container_path), false).is_empty() {
                let loc = Loc::new();
                let rfile = RFile::new_from_decoded(
                    &RFileDecoded::Loc(loc), 0, loc_path
                );
                pack.insert(rfile)?;
            }

            let mut files = pack.files_by_type_and_paths_mut(
                &[FileType::Loc],
                from_ref(&container_path),
                true,
            );

            if let Some(file) = files.first_mut() {
                if let Ok(RFileDecoded::Loc(loc)) = file.decoded_mut() {
                    for (key, text) in entries {
                        loc.data_mut().push(vec![
                            DecodedData::StringU16(key.to_string()),
                            DecodedData::StringU16(text.to_string()),
                            DecodedData::Boolean(true),
                        ]);
                    }
                }
            }

            Ok(container_path)
        }

        // ── macro shorthand ────────────────────────────────────
        macro_rules! row {
            ($($k:expr => $v:expr),* $(,)?) => {{
                let mut m = std::collections::HashMap::new();
                $(m.insert($k, $v.to_string());)*
                m
            }};
        }

        // ── file stem shared across all entries ────────────────
        let stem = format!("ceo_{}", entries.first().map(|e| e.name.as_str()).unwrap_or("builder"));
        let stem = stem.as_str();
        let loc_path = format!("text/ceo/general_items/{}.loc", stem);

        // ── process each entry ─────────────────────────────────
        let mut loc_entries: Vec<(String, String)> = Vec::new();

        for entry in entries {
            let n = &entry.name;
            let element = &entry.element;
            let gender = &entry.gender;
            let is_unique = entry.option == "unique";

            if is_unique {
                // ── UNIQUE PATH ────────────────────────────────

                // ceos — armor
                let p = insert_row(pack, schema, "ceos_tables", stem, &row![
                    "key" => format!("3k_main_ancilliary_armour_{n}_armour_unique"),
                    "exists_in_location" => "character_ceo_manager",
                    "category" => "3k_main_ceo_category_ancillary_armour",
                    "equipped_in_location" => "character_equipment",
                    "priority" => "1",
                    "turns_to_expire" => "0",
                    "point_change_per_turn_if_inactive" => "0",
                    "point_change_per_turn_while_active" => "0",
                    "point_change_per_turn_while_equipped" => "0",
                    "inheritance_chance" => "0",
                    "can_be_looted_post_battle" => "false",
                    "can_be_traded_in_diplomacy" => "false",
                    "can_be_stolen" => "false",
                    "rarity" => "unique",
                    "can_be_unequipped" => "false",
                    "can_be_transferred_if_equipped" => "true",
                    "cannot_reequip_until_next_round_if_unequipped" => "true",
                    "provides_scripted_permissions_on_spawn" => "",
                ])?;
                if !added_paths.contains(&p) { added_paths.push(p); }

                // ceos — career/title
                let p = insert_row(pack, schema, "ceos_tables", stem, &row![
                    "key" => format!("3k_main_ceo_career_historical_{n}"),
                    "exists_in_location" => "character_ceo_manager",
                    "category" => "3k_main_ceo_category_career",
                    "equipped_in_location" => "character_equipment",
                    "priority" => "1",
                    "turns_to_expire" => "0",
                    "point_change_per_turn_if_inactive" => "0",
                    "point_change_per_turn_while_active" => "0",
                    "point_change_per_turn_while_equipped" => "0",
                    "inheritance_chance" => "0",
                    "can_be_looted_post_battle" => "false",
                    "can_be_traded_in_diplomacy" => "false",
                    "can_be_stolen" => "false",
                    "rarity" => "common",
                    "can_be_unequipped" => "false",
                    "can_be_transferred_if_equipped" => "true",
                    "cannot_reequip_until_next_round_if_unequipped" => "true",
                    "provides_scripted_permissions_on_spawn" => "",
                ])?;
                if !added_paths.contains(&p) { added_paths.push(p); }

                // ceo_groups
                let p = insert_row(pack, schema, "ceo_groups_tables", stem, &row![
                    "key" => format!("3k_main_ceo_group_ancillary_armour_character_specific_{n}"),
                ])?;
                if !added_paths.contains(&p) { added_paths.push(p); }

                // ceo_group_ceos
                let grp_key = format!("3k_main_ceo_group_ancillary_armour_character_specific_{n}");
                let armor_key = format!("3k_main_ancilliary_armour_{n}_armour_unique");
                let career_key_grp = format!("3k_main_ceo_career_historical_{n}");
                // armour into character_specific group
                let p = insert_row(pack, schema, "ceo_group_ceos_tables", stem, &row![
                    "ceo_group" => &grp_key,
                    "ceo" => &armor_key,
                    "trigger_weighting" => "0.1",
                    "auto_id" => auto_id("ceo_group_ceos", &format!("{grp_key}|{armor_key}")),
                ])?;
                if !added_paths.contains(&p) { added_paths.push(p); }
                // armour into type_character_specific group
                let p = insert_row(pack, schema, "ceo_group_ceos_tables", stem, &row![
                    "ceo_group" => "3k_main_ceo_group_ancillary_armour_type_character_specific",
                    "ceo" => &armor_key,
                    "trigger_weighting" => "0.1",
                    "auto_id" => auto_id("ceo_group_ceos", &format!("3k_main_ceo_group_ancillary_armour_type_character_specific|{armor_key}")),
                ])?;
                if !added_paths.contains(&p) { added_paths.push(p); }
                // armour into character_all group
                let p = insert_row(pack, schema, "ceo_group_ceos_tables", stem, &row![
                    "ceo_group" => "3k_main_ceo_group_ancillary_armour_character_all",
                    "ceo" => &armor_key,
                    "trigger_weighting" => "0.1",
                    "auto_id" => auto_id("ceo_group_ceos", &format!("3k_main_ceo_group_ancillary_armour_character_all|{armor_key}")),
                ])?;
                if !added_paths.contains(&p) { added_paths.push(p); }
                // career into career_all group
                let p = insert_row(pack, schema, "ceo_group_ceos_tables", stem, &row![
                    "ceo_group" => "3k_main_ceo_group_career_all",
                    "ceo" => &career_key_grp,
                    "trigger_weighting" => "1",
                    "auto_id" => auto_id("ceo_group_ceos", &format!("3k_main_ceo_group_career_all|{career_key_grp}")),
                ])?;
                if !added_paths.contains(&p) { added_paths.push(p); }

                // ceo_permissions
                let perm_key = format!("3k_main_ceo_permissions_ancillary_armour_character_specific_{n}");
                let p = insert_row(pack, schema, "ceo_permissions_tables", stem, &row![
                    "key" => &perm_key,
                ])?;
                if !added_paths.contains(&p) { added_paths.push(p); }

                // ceo_permissions_groups
                let p = insert_row(pack, schema, "ceo_permissions_groups_tables", stem, &row![
                    "permissions" => &perm_key,
                    "group" => &grp_key,
                    "point_gain_enabled_override" => "true",
                    "point_gain_disabled_override" => "false",
                    "state_active_override" => "true",
                    "state_inactive_override" => "false",
                    "auto_id" => auto_id("ceo_permissions_groups", &format!("{perm_key}|{grp_key}")),
                ])?;
                if !added_paths.contains(&p) { added_paths.push(p); }

                // ceo_scripted_permissions
                let scripted_perm_key = format!("3k_main_ceo_permissions_ancillary_armour_character_specific_{n}");
                let p = insert_row(pack, schema, "ceo_scripted_permissions_tables", stem, &row![
                    "key" => &scripted_perm_key,
                    "exists_in_and_provides_permission_to_location" => "character_ceo_manager",
                ])?;
                if !added_paths.contains(&p) { added_paths.push(p); }

                // ceo_scripted_permissions_to_permissions
                let p = insert_row(pack, schema, "ceo_scripted_permissions_to_permissions_tables", stem, &row![
                    "scripted_permissions" => &scripted_perm_key,
                    "permissions" => &perm_key,
                    "auto_id" => auto_id("ceo_scripted_permissions_to_permissions", &format!("{scripted_perm_key}|{perm_key}")),
                ])?;
                if !added_paths.contains(&p) { added_paths.push(p); }

                // ceo_initial_data_stages — two stages
                let stage1_key = format!("3k_main_ceo_initial_data_stage_character_traits_historical_{n}");
                let stage2_key = format!("3k_main_ceo_initial_data_character_historical_{n}_ancillaries");
                for key in &[&stage1_key, &stage2_key] {
                    let p = insert_row(pack, schema, "ceo_initial_data_stages_tables", stem, &row![
                        "key" => *key,
                    ])?;
                    if !added_paths.contains(&p) { added_paths.push(p); }
                }

                // ceo_effect_lists — armor + career
                let effect_list_armor = format!("3k_main_ancilliary_armour_{n}_armour_unique");
                let effect_list_career = format!("3k_main_ceo_career_historical_{n}");
                for key in &[&effect_list_armor, &effect_list_career] {
                    let p = insert_row(pack, schema, "ceo_effect_lists_tables", stem, &row![
                        "key" => *key,
                    ])?;
                    if !added_paths.contains(&p) { added_paths.push(p); }
                }

                // ceo_initial_datas
                let initial_data_key = format!("3k_main_ceo_initial_data_character_historical_{n}");
                let p = insert_row(pack, schema, "ceo_initial_datas_tables", stem, &row![
                    "key" => &initial_data_key,
                    "template_manager" => "character_ceo_manager",
                ])?;
                if !added_paths.contains(&p) { added_paths.push(p); }

                // Element-specific: scripted permissions + equipments + active_ceos + to_stages
                let armor_ceo_key = format!("3k_main_ancilliary_armour_{n}_armour_unique");
                match element.as_str() {
                    "metal" => {
                        let armour_perm_key = format!("3k_main_ceo_permissions_ancillary_armour_character_specific_{n}");
                        for scripted in &[
                            "3k_main_ceo_permissions_ancillary_weapon_character_sword_dual_enable",
                            "3k_main_ceo_permissions_ancillary_weapon_character_axe_dual_enable",
                            "3k_main_ceo_permissions_ancillary_weapon_character_sword_one_handed_enable",
                            "3k_main_ceo_permissions_ancillary_weapon_character_axe_one_handed_enable",
                            "3k_ytr_ceo_permissions_ancillary_weapon_character_mace_dual_enable",
                            armour_perm_key.as_str(),
                        ] {
                            let p = insert_row(pack, schema, "ceo_initial_data_scripted_permissions_tables", stem, &row![
                                "initial_data_stage" => &stage2_key,
                                "scripted_permissions" => *scripted,
                                "auto_id" => auto_id("ceo_initial_data_scripted_permissions", &format!("{stage2_key}|{scripted}")),
                            ])?;
                            if !added_paths.contains(&p) { added_paths.push(p); }
                        }
                        for (category, equipped_ceo) in &[
                            ("3k_main_ceo_category_ancillary_weapon", "3k_main_ancillary_weapon_double_edged_sword_common"),
                            ("3k_main_ceo_category_ancillary_mount",  "3k_main_ancillary_mount_grey_horse"),
                            ("3k_main_ceo_category_ancillary_armour", armor_ceo_key.as_str()),
                        ] {
                            let p = insert_row(pack, schema, "ceo_initial_data_equipments_tables", stem, &row![
                                "initial_data_stage" => &stage2_key,
                                "category" => *category,
                                "equipped_ceo" => *equipped_ceo,
                                "slot_index" => "0",
                                "target" => "character_equipment",
                                "auto_id" => auto_id("ceo_initial_data_equipments", &format!("{stage2_key}|{equipped_ceo}")),
                            ])?;
                            if !added_paths.contains(&p) { added_paths.push(p); }
                        }
                        for active_ceo in &[
                            "3k_main_ancillary_weapon_single_edged_sword_common",
                            "3k_ytr_ancillary_weapon_dual_maces_common",
                            "3k_main_ancillary_weapon_one_handed_axe_common",
                            "3k_main_ancillary_weapon_dual_swords_common",
                            "3k_main_ancillary_weapon_double_edged_sword_common",
                            "3k_main_ancillary_mount_grey_horse",
                        ] {
                            let p = insert_row(pack, schema, "ceo_initial_data_active_ceos_tables", stem, &row![
                                "initial_data_stage" => &stage2_key,
                                "active_ceo" => *active_ceo,
                                "starting_points_delta" => "0",
                                "auto_id" => auto_id("ceo_initial_data_active_ceos", &format!("{stage2_key}|{active_ceo}")),
                            ])?;
                            if !added_paths.contains(&p) { added_paths.push(p); }
                        }
                        // class into stage1
                        let p = insert_row(pack, schema, "ceo_initial_data_active_ceos_tables", stem, &row![
                            "initial_data_stage" => &stage1_key,
                            "active_ceo" => "3k_main_ceo_class_metal",
                            "starting_points_delta" => "0",
                            "auto_id" => auto_id("ceo_initial_data_active_ceos", &format!("{stage1_key}|3k_main_ceo_class_metal")),
                        ])?;
                        if !added_paths.contains(&p) { added_paths.push(p); }
                        // armour active_ceo into stage2
                        let p = insert_row(pack, schema, "ceo_initial_data_active_ceos_tables", stem, &row![
                            "initial_data_stage" => &stage2_key,
                            "active_ceo" => &armor_ceo_key,
                            "starting_points_delta" => "0",
                            "auto_id" => auto_id("ceo_initial_data_active_ceos", &format!("{stage2_key}|{armor_ceo_key}")),
                        ])?;
                        if !added_paths.contains(&p) { added_paths.push(p); }
                        // expanded: sword_and_shield only if expanded
                        if entry.expanded {
                            let p = insert_row(pack, schema, "ceo_initial_data_active_ceos_tables", stem, &row![
                                "initial_data_stage" => &stage2_key,
                                "active_ceo" => "3k_main_ancillary_weapon_sword_and_shield_common",
                                "starting_points_delta" => "0",
                                "auto_id" => auto_id("ceo_initial_data_active_ceos", &format!("{stage2_key}|3k_main_ancillary_weapon_sword_and_shield_common")),
                            ])?;
                            if !added_paths.contains(&p) { added_paths.push(p); }
                        }
                        // armour effect: expertise_mod
                        let p = insert_row(pack, schema, "ceo_effect_list_to_effects_tables", stem, &row![
                            "effect_list" => &effect_list_armor,
                            "effect" => "3k_main_effect_character_attribute_expertise_mod",
                            "value" => "18",
                            "effect_scope" => "character_to_character_own",
                            "optional_only_in_game_mode" => "",
                            "auto_id" => auto_id("ceo_effect_list_to_effects", &format!("{effect_list_armor}|expertise")),
                        ])?;
                        if !added_paths.contains(&p) { added_paths.push(p); }
                        for (id_stage, stage_num) in &[
                            ("3k_main_ceo_initial_data_stage_character_childhood_metal", 17i32),
                            ("3k_main_ceo_initial_data_equipment_permissions_unique_metal", 4i32),
                        ] {
                            let p = insert_row(pack, schema, "ceo_initial_data_to_stages_tables", stem, &row![
                                "ceo_initial_data" => &initial_data_key,
                                "initial_data_stage" => *id_stage,
                                "stage" => stage_num,
                            ])?;
                            if !added_paths.contains(&p) { added_paths.push(p); }
                        }
                    },
                    "water" => {
                        for scripted in &[
                            "3k_main_ceo_permissions_ancillary_weapon_character_sword_one_handed_enable",
                            &format!("3k_main_ceo_permissions_ancillary_armour_character_specific_{n}"),
                        ] {
                            let p = insert_row(pack, schema, "ceo_initial_data_scripted_permissions_tables", stem, &row![
                                "initial_data_stage" => &stage2_key,
                                "scripted_permissions" => *scripted,
                                "auto_id" => auto_id("ceo_initial_data_scripted_permissions", &format!("{stage2_key}|{scripted}")),
                            ])?;
                            if !added_paths.contains(&p) { added_paths.push(p); }
                        }
                        for (category, equipped_ceo) in &[
                            ("3k_main_ceo_category_ancillary_mount",  "3k_main_ancillary_mount_black_horse"),
                            ("3k_main_ceo_category_ancillary_weapon", "3k_main_ancillary_weapon_single_edged_sword_common"),
                            ("3k_main_ceo_category_ancillary_armour", armor_ceo_key.as_str()),
                        ] {
                            let p = insert_row(pack, schema, "ceo_initial_data_equipments_tables", stem, &row![
                                "initial_data_stage" => &stage2_key,
                                "category" => *category,
                                "equipped_ceo" => *equipped_ceo,
                                "slot_index" => "0",
                                "target" => "character_equipment",
                                "auto_id" => auto_id("ceo_initial_data_equipments", &format!("{stage2_key}|{equipped_ceo}")),
                            ])?;
                            if !added_paths.contains(&p) { added_paths.push(p); }
                        }
                        for active_ceo in &[
                            "3k_main_ancillary_weapon_single_edged_sword_common",
                            "3k_main_ancillary_weapon_dual_swords_common",
                            "3k_main_ancillary_weapon_double_edged_sword_common",
                            "3k_main_ancillary_mount_black_horse",
                        ] {
                            let p = insert_row(pack, schema, "ceo_initial_data_active_ceos_tables", stem, &row![
                                "initial_data_stage" => &stage2_key,
                                "active_ceo" => *active_ceo,
                                "starting_points_delta" => "0",
                                "auto_id" => auto_id("ceo_initial_data_active_ceos", &format!("{stage2_key}|{active_ceo}")),
                            ])?;
                            if !added_paths.contains(&p) { added_paths.push(p); }
                        }
                        // armour active_ceo into stage2
                        let p = insert_row(pack, schema, "ceo_initial_data_active_ceos_tables", stem, &row![
                            "initial_data_stage" => &stage2_key,
                            "active_ceo" => &armor_ceo_key,
                            "starting_points_delta" => "0",
                            "auto_id" => auto_id("ceo_initial_data_active_ceos", &format!("{stage2_key}|{armor_ceo_key}")),
                        ])?;
                        if !added_paths.contains(&p) { added_paths.push(p); }
                        // class into stage1
                        let p = insert_row(pack, schema, "ceo_initial_data_active_ceos_tables", stem, &row![
                            "initial_data_stage" => &stage1_key,
                            "active_ceo" => "3k_main_ceo_class_water",
                            "starting_points_delta" => "0",
                            "auto_id" => auto_id("ceo_initial_data_active_ceos", &format!("{stage1_key}|3k_main_ceo_class_water")),
                        ])?;
                        if !added_paths.contains(&p) { added_paths.push(p); }
                        // armour effect: cunning_mod
                        let p = insert_row(pack, schema, "ceo_effect_list_to_effects_tables", stem, &row![
                            "effect_list" => &effect_list_armor,
                            "effect" => "3k_main_effect_character_attribute_cunning_mod",
                            "value" => "18",
                            "effect_scope" => "character_to_character_own",
                            "optional_only_in_game_mode" => "",
                            "auto_id" => auto_id("ceo_effect_list_to_effects", &format!("{effect_list_armor}|cunning")),
                        ])?;
                        if !added_paths.contains(&p) { added_paths.push(p); }
                        for (id_stage, stage_num) in &[
                            ("3k_main_ceo_initial_data_stage_character_childhood_water", 17i32),
                            ("3k_main_ceo_initial_data_equipment_permissions_unique_water", 4i32),
                        ] {
                            let p = insert_row(pack, schema, "ceo_initial_data_to_stages_tables", stem, &row![
                                "ceo_initial_data" => &initial_data_key,
                                "initial_data_stage" => *id_stage,
                                "stage" => stage_num,
                            ])?;
                            if !added_paths.contains(&p) { added_paths.push(p); }
                        }
                    },
                    "earth" => {
                        for scripted in &[
                            "3k_main_ceo_permissions_ancillary_weapon_character_axe_dual_enable",
                            "3k_main_ceo_permissions_ancillary_weapon_character_axe_one_handed_enable",
                            &format!("3k_main_ceo_permissions_ancillary_armour_character_specific_{n}"),
                            "3k_main_ceo_permissions_ancillary_weapon_character_sword_one_handed_enable",
                            "3k_main_ceo_permissions_ancillary_weapon_character_sword_dual_enable",
                        ] {
                            let p = insert_row(pack, schema, "ceo_initial_data_scripted_permissions_tables", stem, &row![
                                "initial_data_stage" => &stage2_key,
                                "scripted_permissions" => *scripted,
                                "auto_id" => auto_id("ceo_initial_data_scripted_permissions", &format!("{stage2_key}|{scripted}")),
                            ])?;
                            if !added_paths.contains(&p) { added_paths.push(p); }
                        }
                        for (category, equipped_ceo) in &[
                            ("3k_main_ceo_category_ancillary_weapon", "3k_main_ancillary_weapon_single_edged_sword_common"),
                            ("3k_main_ceo_category_ancillary_mount",  "3k_main_ancillary_mount_white_horse"),
                            ("3k_main_ceo_category_ancillary_armour", armor_ceo_key.as_str()),
                        ] {
                            let p = insert_row(pack, schema, "ceo_initial_data_equipments_tables", stem, &row![
                                "initial_data_stage" => &stage2_key,
                                "category" => *category,
                                "equipped_ceo" => *equipped_ceo,
                                "slot_index" => "0",
                                "target" => "character_equipment",
                                "auto_id" => auto_id("ceo_initial_data_equipments", &format!("{stage2_key}|{equipped_ceo}")),
                            ])?;
                            if !added_paths.contains(&p) { added_paths.push(p); }
                        }
                        for active_ceo in &[
                            "3k_main_ancillary_weapon_one_handed_axe_common",
                            "3k_main_ancillary_weapon_dual_swords_common",
                            "3k_main_ancillary_mount_white_horse",
                            "3k_main_ancillary_weapon_single_edged_sword_common",
                            "3k_main_ancillary_weapon_double_edged_sword_common",
                        ] {
                            let p = insert_row(pack, schema, "ceo_initial_data_active_ceos_tables", stem, &row![
                                "initial_data_stage" => &stage2_key,
                                "active_ceo" => *active_ceo,
                                "starting_points_delta" => "0",
                                "auto_id" => auto_id("ceo_initial_data_active_ceos", &format!("{stage2_key}|{active_ceo}")),
                            ])?;
                            if !added_paths.contains(&p) { added_paths.push(p); }
                        }
                        // armour active_ceo into stage2
                        let p = insert_row(pack, schema, "ceo_initial_data_active_ceos_tables", stem, &row![
                            "initial_data_stage" => &stage2_key,
                            "active_ceo" => &armor_ceo_key,
                            "starting_points_delta" => "0",
                            "auto_id" => auto_id("ceo_initial_data_active_ceos", &format!("{stage2_key}|{armor_ceo_key}")),
                        ])?;
                        if !added_paths.contains(&p) { added_paths.push(p); }
                        // class into stage1
                        let p = insert_row(pack, schema, "ceo_initial_data_active_ceos_tables", stem, &row![
                            "initial_data_stage" => &stage1_key,
                            "active_ceo" => "3k_main_ceo_class_earth",
                            "starting_points_delta" => "0",
                            "auto_id" => auto_id("ceo_initial_data_active_ceos", &format!("{stage1_key}|3k_main_ceo_class_earth")),
                        ])?;
                        if !added_paths.contains(&p) { added_paths.push(p); }
                        // expanded: sword_and_shield
                        if entry.expanded {
                            let p = insert_row(pack, schema, "ceo_initial_data_active_ceos_tables", stem, &row![
                                "initial_data_stage" => &stage2_key,
                                "active_ceo" => "3k_main_ancillary_weapon_sword_and_shield_common",
                                "starting_points_delta" => "0",
                                "auto_id" => auto_id("ceo_initial_data_active_ceos", &format!("{stage2_key}|3k_main_ancillary_weapon_sword_and_shield_common")),
                            ])?;
                            if !added_paths.contains(&p) { added_paths.push(p); }
                        }
                        // armour effect: authority_mod
                        let p = insert_row(pack, schema, "ceo_effect_list_to_effects_tables", stem, &row![
                            "effect_list" => &effect_list_armor,
                            "effect" => "3k_main_effect_character_attribute_authority_mod",
                            "value" => "18",
                            "effect_scope" => "character_to_character_own",
                            "optional_only_in_game_mode" => "",
                            "auto_id" => auto_id("ceo_effect_list_to_effects", &format!("{effect_list_armor}|authority")),
                        ])?;
                        if !added_paths.contains(&p) { added_paths.push(p); }
                        for (id_stage, stage_num) in &[
                            ("3k_main_ceo_initial_data_stage_character_childhood_earth", 17i32),
                            ("3k_main_ceo_initial_data_equipment_permissions_unique_earth", 4i32),
                        ] {
                            let p = insert_row(pack, schema, "ceo_initial_data_to_stages_tables", stem, &row![
                                "ceo_initial_data" => &initial_data_key,
                                "initial_data_stage" => *id_stage,
                                "stage" => stage_num,
                            ])?;
                            if !added_paths.contains(&p) { added_paths.push(p); }
                        }
                    },
                    "fire" => {
                        for scripted in &[
                            "3k_main_ceo_permissions_ancillary_weapon_character_axe_two_handed_enable",
                            "3k_main_ceo_permissions_ancillary_weapon_character_spear_two_handed_long_enable",
                            &format!("3k_main_ceo_permissions_ancillary_armour_character_specific_{n}"),
                            "3k_main_ceo_permissions_ancillary_weapon_character_spear_two_handed_enable",
                            "3k_main_ceo_permissions_ancillary_weapon_character_axe_two_handed_enable",
                        ] {
                            let p = insert_row(pack, schema, "ceo_initial_data_scripted_permissions_tables", stem, &row![
                                "initial_data_stage" => &stage2_key,
                                "scripted_permissions" => *scripted,
                                "auto_id" => auto_id("ceo_initial_data_scripted_permissions", &format!("{stage2_key}|{scripted}")),
                            ])?;
                            if !added_paths.contains(&p) { added_paths.push(p); }
                        }
                        for (category, equipped_ceo) in &[
                            ("3k_main_ceo_category_ancillary_mount",  "3k_main_ancillary_mount_red_horse"),
                            ("3k_main_ceo_category_ancillary_weapon", "3k_main_ancillary_weapon_two_handed_spear_common"),
                            ("3k_main_ceo_category_ancillary_armour", armor_ceo_key.as_str()),
                        ] {
                            let p = insert_row(pack, schema, "ceo_initial_data_equipments_tables", stem, &row![
                                "initial_data_stage" => &stage2_key,
                                "category" => *category,
                                "equipped_ceo" => *equipped_ceo,
                                "slot_index" => "0",
                                "target" => "character_equipment",
                                "auto_id" => auto_id("ceo_initial_data_equipments", &format!("{stage2_key}|{equipped_ceo}")),
                            ])?;
                            if !added_paths.contains(&p) { added_paths.push(p); }
                        }
                        for active_ceo in &[
                            "3k_main_ancillary_weapon_hook_sickle_sabre_common",
                            "3k_main_ancillary_weapon_two_handed_axe_common",
                            "3k_main_ancillary_weapon_two_handed_spear_common",
                            "3k_main_ancillary_weapon_halberd_common",
                            "3k_main_ancillary_mount_red_horse",
                        ] {
                            let p = insert_row(pack, schema, "ceo_initial_data_active_ceos_tables", stem, &row![
                                "initial_data_stage" => &stage2_key,
                                "active_ceo" => *active_ceo,
                                "starting_points_delta" => "0",
                                "auto_id" => auto_id("ceo_initial_data_active_ceos", &format!("{stage2_key}|{active_ceo}")),
                            ])?;
                            if !added_paths.contains(&p) { added_paths.push(p); }
                        }
                        // armour active_ceo into stage2
                        let p = insert_row(pack, schema, "ceo_initial_data_active_ceos_tables", stem, &row![
                            "initial_data_stage" => &stage2_key,
                            "active_ceo" => &armor_ceo_key,
                            "starting_points_delta" => "0",
                            "auto_id" => auto_id("ceo_initial_data_active_ceos", &format!("{stage2_key}|{armor_ceo_key}")),
                        ])?;
                        if !added_paths.contains(&p) { added_paths.push(p); }
                        // class into stage1
                        let p = insert_row(pack, schema, "ceo_initial_data_active_ceos_tables", stem, &row![
                            "initial_data_stage" => &stage1_key,
                            "active_ceo" => "3k_main_ceo_class_fire",
                            "starting_points_delta" => "0",
                            "auto_id" => auto_id("ceo_initial_data_active_ceos", &format!("{stage1_key}|3k_main_ceo_class_fire")),
                        ])?;
                        if !added_paths.contains(&p) { added_paths.push(p); }
                        // armour effect: instinct_mod
                        let p = insert_row(pack, schema, "ceo_effect_list_to_effects_tables", stem, &row![
                            "effect_list" => &effect_list_armor,
                            "effect" => "3k_main_effect_character_attribute_instinct_mod",
                            "value" => "18",
                            "effect_scope" => "character_to_character_own",
                            "optional_only_in_game_mode" => "",
                            "auto_id" => auto_id("ceo_effect_list_to_effects", &format!("{effect_list_armor}|instinct")),
                        ])?;
                        if !added_paths.contains(&p) { added_paths.push(p); }
                        for (id_stage, stage_num) in &[
                            ("3k_main_ceo_initial_data_stage_character_childhood_fire", 17i32),
                            ("3k_main_ceo_initial_data_equipment_permissions_unique_fire", 4i32),
                        ] {
                            let p = insert_row(pack, schema, "ceo_initial_data_to_stages_tables", stem, &row![
                                "ceo_initial_data" => &initial_data_key,
                                "initial_data_stage" => *id_stage,
                                "stage" => stage_num,
                            ])?;
                            if !added_paths.contains(&p) { added_paths.push(p); }
                        }
                    },
                    _ => { // wood
                        for scripted in &[
                            &format!("3k_main_ceo_permissions_ancillary_armour_character_specific_{n}"),
                            "3k_main_ceo_permissions_ancillary_weapon_character_spear_two_handed_enable",
                            "3k_main_ceo_permissions_ancillary_weapon_character_spear_two_handed_long_enable",
                            "3k_ytr_ceo_permissions_ancillary_weapon_character_staff_two_handed_enable",
                            "3k_ytr_ceo_permissions_ancillary_weapon_character_mace_two_handed_enable",
                        ] {
                            let p = insert_row(pack, schema, "ceo_initial_data_scripted_permissions_tables", stem, &row![
                                "initial_data_stage" => &stage2_key,
                                "scripted_permissions" => *scripted,
                                "auto_id" => auto_id("ceo_initial_data_scripted_permissions", &format!("{stage2_key}|{scripted}")),
                            ])?;
                            if !added_paths.contains(&p) { added_paths.push(p); }
                        }
                        for (category, equipped_ceo) in &[
                            ("3k_main_ceo_category_ancillary_mount",  "3k_main_ancillary_mount_brown_horse"),
                            ("3k_main_ceo_category_ancillary_weapon", "3k_main_ancillary_weapon_two_handed_spear_common"),
                            ("3k_main_ceo_category_ancillary_armour", armor_ceo_key.as_str()),
                        ] {
                            let p = insert_row(pack, schema, "ceo_initial_data_equipments_tables", stem, &row![
                                "initial_data_stage" => &stage2_key,
                                "category" => *category,
                                "equipped_ceo" => *equipped_ceo,
                                "slot_index" => "0",
                                "target" => "character_equipment",
                                "auto_id" => auto_id("ceo_initial_data_equipments", &format!("{stage2_key}|{equipped_ceo}")),
                            ])?;
                            if !added_paths.contains(&p) { added_paths.push(p); }
                        }
                        for active_ceo in &[
                            "3k_ytr_ancillary_weapon_2h_ball_mace_common",
                            "3k_main_ancillary_weapon_hook_sickle_sabre_common",
                            "3k_main_ancillary_mount_brown_horse",
                            "3k_main_ancillary_weapon_two_handed_spear_common",
                            "3k_main_ancillary_weapon_halberd_common",
                        ] {
                            let p = insert_row(pack, schema, "ceo_initial_data_active_ceos_tables", stem, &row![
                                "initial_data_stage" => &stage2_key,
                                "active_ceo" => *active_ceo,
                                "starting_points_delta" => "0",
                                "auto_id" => auto_id("ceo_initial_data_active_ceos", &format!("{stage2_key}|{active_ceo}")),
                            ])?;
                            if !added_paths.contains(&p) { added_paths.push(p); }
                        }
                        // armour active_ceo into stage2
                        let p = insert_row(pack, schema, "ceo_initial_data_active_ceos_tables", stem, &row![
                            "initial_data_stage" => &stage2_key,
                            "active_ceo" => &armor_ceo_key,
                            "starting_points_delta" => "0",
                            "auto_id" => auto_id("ceo_initial_data_active_ceos", &format!("{stage2_key}|{armor_ceo_key}")),
                        ])?;
                        if !added_paths.contains(&p) { added_paths.push(p); }
                        // class into stage1
                        let p = insert_row(pack, schema, "ceo_initial_data_active_ceos_tables", stem, &row![
                            "initial_data_stage" => &stage1_key,
                            "active_ceo" => "3k_main_ceo_class_wood",
                            "starting_points_delta" => "0",
                            "auto_id" => auto_id("ceo_initial_data_active_ceos", &format!("{stage1_key}|3k_main_ceo_class_wood")),
                        ])?;
                        if !added_paths.contains(&p) { added_paths.push(p); }
                        // armour effect: resolve_mod
                        let p = insert_row(pack, schema, "ceo_effect_list_to_effects_tables", stem, &row![
                            "effect_list" => &effect_list_armor,
                            "effect" => "3k_main_effect_character_attribute_resolve_mod",
                            "value" => "18",
                            "effect_scope" => "character_to_character_own",
                            "optional_only_in_game_mode" => "",
                            "auto_id" => auto_id("ceo_effect_list_to_effects", &format!("{effect_list_armor}|resolve")),
                        ])?;
                        if !added_paths.contains(&p) { added_paths.push(p); }
                        for (id_stage, stage_num) in &[
                            ("3k_main_ceo_initial_data_stage_character_childhood_wood", 17i32),
                            ("3k_main_ceo_initial_data_equipment_permissions_unique_wood", 4i32),
                        ] {
                            let p = insert_row(pack, schema, "ceo_initial_data_to_stages_tables", stem, &row![
                                "ceo_initial_data" => &initial_data_key,
                                "initial_data_stage" => *id_stage,
                                "stage" => stage_num,
                            ])?;
                            if !added_paths.contains(&p) { added_paths.push(p); }
                        }
                    },
                }
                // Gender
                let gender_stage = if gender == "male" {
                    "3k_main_ceo_initial_data_stage_character_gender_male"
                } else {
                    "3k_main_ceo_initial_data_stage_character_gender_female"
                };
                let p = insert_row(pack, schema, "ceo_initial_data_to_stages_tables", stem, &row![
                    "ceo_initial_data" => &initial_data_key,
                    "initial_data_stage" => gender_stage,
                    "stage" => "13",
                ])?;
                if !added_paths.contains(&p) { added_paths.push(p); }

                // Shared unique records after element/gender

                // Link the _ancillaries stage (stage2_key) at stage order 3
                let p = insert_row(pack, schema, "ceo_initial_data_to_stages_tables", stem, &row![
                    "ceo_initial_data" => &initial_data_key,
                    "initial_data_stage" => &stage2_key,
                    "stage" => "3",
                ])?;
                if !added_paths.contains(&p) { added_paths.push(p); }

                let p = insert_row(pack, schema, "ceo_initial_data_active_ceos_tables", stem, &row![
                    "initial_data_stage" => &stage1_key,
                    "active_ceo" => &format!("3k_main_ceo_career_historical_{n}"),
                    "starting_points_delta" => "0",
                    "auto_id" => auto_id("ceo_initial_data_active_ceos", &format!("{stage1_key}|3k_main_ceo_career_historical_{n}")),
                ])?;
                if !added_paths.contains(&p) { added_paths.push(p); }

                for (id_stage, stage_num) in &[
                    ("3k_main_initial_data_character_ancillaries_global", 2i32),
                    ("3k_dlc04_ceo_initial_data_character_give_political_support_random", 21i32),
                    ("3k_main_ceo_initial_data_stage_character_wealth_random", 15i32),
                    ("3k_main_ceo_initial_data_stage_character_traits_shared_global_permissions", 10i32),
                    ("3k_main_ceo_initial_data_stage_character_protagonist", 14i32),
                ] {
                    let p = insert_row(pack, schema, "ceo_initial_data_to_stages_tables", stem, &row![
                        "ceo_initial_data" => &initial_data_key,
                        "initial_data_stage" => *id_stage,
                        "stage" => stage_num,
                    ])?;
                    if !added_paths.contains(&p) { added_paths.push(p); }
                }
                let p = insert_row(pack, schema, "ceo_initial_data_to_stages_tables", stem, &row![
                    "ceo_initial_data" => &initial_data_key,
                    "initial_data_stage" => &stage1_key,
                    "stage" => "11",
                ])?;
                if !added_paths.contains(&p) { added_paths.push(p); }

                // ceo_thresholds
                for (ceo_ref, key_val) in &[
                    (format!("3k_main_ceo_career_historical_{n}"), format!("3k_main_ceo_career_historical_{n}")),
                    (format!("3k_main_ancilliary_armour_{n}_armour_unique"), format!("3k_main_ancilliary_armour_{n}_armour_unique")),
                ] {
                    let p = insert_row(pack, schema, "ceo_thresholds_tables", stem, &row![
                        "key" => key_val,
                        "ceo" => ceo_ref,
                        "point_threshold_to_activate" => "1",
                        "point_theshold_to_destroy" => "0",
                        "starting_points" => "1",
                        "max_points" => "1",
                        "resets_to_starting_points_when_deactivated" => "false",
                    ])?;
                    if !added_paths.contains(&p) { added_paths.push(p); }
                }

                // ceo_nodes — career
                let p = insert_row(pack, schema, "ceo_nodes_tables", stem, &row![
                    "key" => format!("3k_main_ceo_career_historical_{n}"),
                    "ceo_effect_list" => &effect_list_career,
                    "title" => "placeholder",
                    "description" => "placeholder",
                    "icon_path" => "",
                    "opinion_topic_modifier" => "",
                    "point_change_per_turn_if_active" => "0",
                ])?;
                if !added_paths.contains(&p) { added_paths.push(p); }
                // ceo_nodes — armour (with element-specific icon)
                let armour_icon = format!("armours/3k_main_ancillary_{}_armour_unique.png", element);
                let p = insert_row(pack, schema, "ceo_nodes_tables", stem, &row![
                    "key" => format!("3k_main_ancilliary_armour_{n}_armour_unique"),
                    "ceo_effect_list" => &effect_list_armor,
                    "title" => "placeholder",
                    "description" => "placeholder",
                    "icon_path" => &armour_icon,
                    "opinion_topic_modifier" => "",
                    "point_change_per_turn_if_active" => "0",
                ])?;
                if !added_paths.contains(&p) { added_paths.push(p); }

                // ceo_threshold_nodes
                for (node_key, threshold_key) in &[
                    (format!("3k_main_ceo_career_historical_{n}"), format!("3k_main_ceo_career_historical_{n}")),
                    (format!("3k_main_ancilliary_armour_{n}_armour_unique"), format!("3k_main_ancilliary_armour_{n}_armour_unique")),
                ] {
                    let p = insert_row(pack, schema, "ceo_threshold_nodes_tables", stem, &row![
                        "ceo_threshold" => threshold_key,
                        "ceo_node" => node_key,
                        "points_threshold_to_activate_node" => "1",
                        "can_downgrade_to_previous_node" => "false",
                        "auto_id" => auto_id("ceo_threshold_nodes", &format!("{threshold_key}|{node_key}")),
                    ])?;
                    if !added_paths.contains(&p) { added_paths.push(p); }
                }

                // ceo_effect_list_to_effects — dummy subcategory for armour
                let p = insert_row(pack, schema, "ceo_effect_list_to_effects_tables", stem, &row![
                    "effect_list" => &effect_list_armor,
                    "effect" => "3k_dummy_effect_ceo_subcategory_armour_unique",
                    "value" => "0",
                    "effect_scope" => "character_to_character_own",
                    "optional_only_in_game_mode" => "",
                    "auto_id" => auto_id("ceo_effect_list_to_effects", &format!("{effect_list_armor}|dummy_subcategory")),
                ])?;
                if !added_paths.contains(&p) { added_paths.push(p); }

                // ceo_effect_list_to_effects (career: wealth + lives)
                for (effect, val) in &[
                    ("3k_main_character_wealth", "2"),
                    ("3k_main_effect_character_num_lives", "1"),
                ] {
                    let p = insert_row(pack, schema, "ceo_effect_list_to_effects_tables", stem, &row![
                        "effect_list" => &effect_list_career,
                        "effect" => *effect,
                        "value" => *val,
                        "effect_scope" => "character_to_character_own",
                        "optional_only_in_game_mode" => "",
                        "auto_id" => auto_id("ceo_effect_list_to_effects", &format!("{effect_list_career}|{effect}")),
                    ])?;
                    if !added_paths.contains(&p) { added_paths.push(p); }
                }

                // Traits
                for (trait_uuid, trait_key) in &entry.traits {
                    let p = insert_row(pack, schema, "ceo_initial_data_active_ceos_tables", stem, &row![
                        "initial_data_stage" => &stage1_key,
                        "active_ceo" => trait_key.as_str(),
                        "starting_points_delta" => "0",
                        "auto_id" => auto_id("ceo_initial_data_active_ceos", &format!("{stage1_key}|{trait_key}|{trait_uuid}")),
                    ])?;
                    if !added_paths.contains(&p) { added_paths.push(p); }
                }

                // ceos_to_equipment_variants
                let p = insert_row(pack, schema, "ceos_to_equipment_variants_tables", stem, &row![
                    "ceos_key" => &format!("3k_main_ancilliary_armour_{n}_armour_unique"),
                    "game_mode" => "",
                    "armour" => "3k_ytr_hero_scholar_unique",
                    "male_vmd" => "",
                    "female_vmd" => "",
                    "mount" => "",
                    "primary_melee_weapon" => "",
                    "primary_missile_weapon" => "",
                    "shield" => "",
                    "man_animation" => "",
                    "mount_animation" => "",
                    "secondary_weapon_animation" => "",
                    "remap_general_unit_to_hero_unit" => "false",
                    "priority" => "1",
                    "autonomous_rider_group" => "",
                    "ground_type_stat_effect_group" => "",
                ])?;
                if !added_paths.contains(&p) { added_paths.push(p); }

                // ceo_template_manager_ceo_limits — limit unique armour to 1 globally
                let p = insert_row(pack, schema, "ceo_template_manager_ceo_limits_tables", stem, &row![
                    "ceo_to_limit" => &format!("3k_main_ancilliary_armour_{n}_armour_unique"),
                    "template_manager" => "3k_main_ceo_template_manager_world_generic",
                    "max_limit_that_can_exist_at_once" => "1",
                    "scoped_limit_or_local_only_limit" => "true",
                    "ceo_category_to_limit" => "",
                    "ceo_node_to_limit" => "",
                    "auto_id" => auto_id("ceo_template_manager_ceo_limits", &format!("3k_main_ancilliary_armour_{n}_armour_unique")),
                ])?;
                if !added_paths.contains(&p) { added_paths.push(p); }

                // Loc entries for unique
                let human = n.replace("_ironic", "")
                    .split('_')
                    .map(|w: &str| {
                        let mut c = w.chars();
                        c.next().map(|f: char| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                loc_entries.push((format!("ceo_nodes_title_3k_main_ceo_career_historical_{n}"), "PLACEHOLDER".into()));
                loc_entries.push((format!("ceo_nodes_description_3k_main_ceo_career_historical_{n}"), "PLACEHOLDER".into()));
                loc_entries.push((format!("ceo_nodes_title_3k_main_ancilliary_armour_{n}_armour_unique"), format!("{human}'s Armour")));
                loc_entries.push((format!("ceo_nodes_description_3k_main_ancilliary_armour_{n}_armour_unique"), "The perfect weight and fit, tailored for this warrior of class and distinction.".into()));

            } else {
                // ── TITLE PATH ────────────────────────────────

                // ceos
                let p = insert_row(pack, schema, "ceos_tables", stem, &row![
                    "key" => format!("3k_main_ceo_career_historical_{n}"),
                    "exists_in_location" => "character_ceo_manager",
                    "category" => "3k_main_ceo_category_career",
                    "equipped_in_location" => "character_equipment",
                    "priority" => "1",
                    "turns_to_expire" => "0",
                    "point_change_per_turn_if_inactive" => "0",
                    "point_change_per_turn_while_active" => "0",
                    "point_change_per_turn_while_equipped" => "0",
                    "inheritance_chance" => "0",
                    "can_be_looted_post_battle" => "false",
                    "can_be_traded_in_diplomacy" => "false",
                    "can_be_stolen" => "false",
                    "rarity" => "common",
                    "can_be_unequipped" => "false",
                    "can_be_transferred_if_equipped" => "true",
                    "cannot_reequip_until_next_round_if_unequipped" => "true",
                    "provides_scripted_permissions_on_spawn" => "",
                ])?;
                if !added_paths.contains(&p) { added_paths.push(p); }

                // ceo_group_ceos — career into career_all
                let career_key_title = format!("3k_main_ceo_career_historical_{n}");
                let p = insert_row(pack, schema, "ceo_group_ceos_tables", stem, &row![
                    "ceo_group" => "3k_main_ceo_group_career_all",
                    "ceo" => &career_key_title,
                    "trigger_weighting" => "1",
                    "auto_id" => auto_id("ceo_group_ceos", &format!("3k_main_ceo_group_career_all|{career_key_title}")),
                ])?;
                if !added_paths.contains(&p) { added_paths.push(p); }

                let stage1_key = format!("3k_main_ceo_initial_data_stage_character_traits_historical_{n}");
                let p = insert_row(pack, schema, "ceo_initial_data_stages_tables", stem, &row![
                    "key" => &stage1_key,
                ])?;
                if !added_paths.contains(&p) { added_paths.push(p); }

                let effect_list_key = format!("3k_main_ceo_career_historical_{n}");
                let p = insert_row(pack, schema, "ceo_effect_lists_tables", stem, &row![
                    "key" => &effect_list_key,
                ])?;
                if !added_paths.contains(&p) { added_paths.push(p); }

                let initial_data_key = format!("3k_main_ceo_initial_data_character_historical_{n}");
                let p = insert_row(pack, schema, "ceo_initial_datas_tables", stem, &row![
                    "key" => &initial_data_key,
                    "template_manager" => "character_ceo_manager",
                ])?;
                if !added_paths.contains(&p) { added_paths.push(p); }

                // Element branch (title)
                let (childhood_stage, equipment_stage, class_ceo, generic_stage, generic_stage_num) = match element.as_str() {
                    "metal" => ("3k_main_ceo_initial_data_stage_character_childhood_metal", "3k_main_ceo_initial_data_equipment_permissions_title_metal", "3k_main_ceo_class_metal", "3k_main_ceo_initial_data_character_generic_metal_ancillaries_01", 3i32),
                    "earth" => ("3k_main_ceo_initial_data_stage_character_childhood_earth", "3k_main_ceo_initial_data_equipment_permissions_title_earth", "3k_main_ceo_class_earth", "3k_main_ceo_initial_data_character_generic_earth_ancillaries_01", 3i32),
                    "water" => ("3k_main_ceo_initial_data_stage_character_childhood_water", "3k_main_ceo_initial_data_equipment_permissions_title_water", "3k_main_ceo_class_water", "3k_main_ceo_initial_data_character_generic_water_ancillaries_01", 3i32),
                    "fire"  => ("3k_main_ceo_initial_data_stage_character_childhood_fire",  "3k_main_ceo_initial_data_equipment_permissions_title_fire",  "3k_main_ceo_class_fire",  "3k_main_ceo_initial_data_character_generic_fire_ancillaries_01",  3i32),
                    _       => ("3k_main_ceo_initial_data_stage_character_childhood_wood",  "3k_main_ceo_initial_data_equipment_permissions_title_wood",  "3k_main_ceo_class_wood",  "3k_main_ceo_initial_data_character_generic_wood_ancillaries_01",  3i32),
                };

                for (id_stage, stage_num) in &[
                    (childhood_stage, 17i32),
                    (equipment_stage, 4i32),
                ] {
                    let p = insert_row(pack, schema, "ceo_initial_data_to_stages_tables", stem, &row![
                        "ceo_initial_data" => &initial_data_key,
                        "initial_data_stage" => *id_stage,
                        "stage" => stage_num,
                    ])?;
                    if !added_paths.contains(&p) { added_paths.push(p); }
                }
                let p = insert_row(pack, schema, "ceo_initial_data_active_ceos_tables", stem, &row![
                    "initial_data_stage" => &stage1_key,
                    "active_ceo" => class_ceo,
                    "starting_points_delta" => "0",
                    "auto_id" => auto_id("ceo_initial_data_active_ceos", &format!("{stage1_key}|{class_ceo}")),
                ])?;
                if !added_paths.contains(&p) { added_paths.push(p); }
                let p = insert_row(pack, schema, "ceo_initial_data_to_stages_tables", stem, &row![
                    "ceo_initial_data" => &initial_data_key,
                    "initial_data_stage" => generic_stage,
                    "stage" => &generic_stage_num,
                ])?;
                if !added_paths.contains(&p) { added_paths.push(p); }

                // Gender
                let gender_stage = if gender == "male" {
                    "3k_main_ceo_initial_data_stage_character_gender_male"
                } else {
                    "3k_main_ceo_initial_data_stage_character_gender_female"
                };
                let p = insert_row(pack, schema, "ceo_initial_data_to_stages_tables", stem, &row![
                    "ceo_initial_data" => &initial_data_key,
                    "initial_data_stage" => gender_stage,
                    "stage" => "13",
                ])?;
                if !added_paths.contains(&p) { added_paths.push(p); }

                // Generic shared (title)
                let career_key = format!("3k_main_ceo_career_historical_{n}");
                let p = insert_row(pack, schema, "ceo_initial_data_active_ceos_tables", stem, &row![
                    "initial_data_stage" => &stage1_key,
                    "active_ceo" => &career_key,
                    "starting_points_delta" => "0",
                    "auto_id" => auto_id("ceo_initial_data_active_ceos", &format!("{stage1_key}|{career_key}")),
                ])?;
                if !added_paths.contains(&p) { added_paths.push(p); }

                for (id_stage, stage_num) in &[
                    ("3k_main_initial_data_character_ancillaries_global", 2i32),
                    ("3k_dlc04_ceo_initial_data_character_give_political_support_random", 21i32),
                    ("3k_main_ceo_initial_data_stage_character_wealth_random", 15i32),
                    ("3k_main_ceo_initial_data_stage_character_traits_shared_global_permissions", 10i32),
                    ("3k_main_ceo_initial_data_stage_character_protagonist", 14i32),
                ] {
                    let p = insert_row(pack, schema, "ceo_initial_data_to_stages_tables", stem, &row![
                        "ceo_initial_data" => &initial_data_key,
                        "initial_data_stage" => *id_stage,
                        "stage" => stage_num,
                    ])?;
                    if !added_paths.contains(&p) { added_paths.push(p); }
                }
                let p = insert_row(pack, schema, "ceo_initial_data_to_stages_tables", stem, &row![
                    "ceo_initial_data" => &initial_data_key,
                    "initial_data_stage" => &stage1_key,
                    "stage" => "11",
                ])?;
                if !added_paths.contains(&p) { added_paths.push(p); }

                // ceo_thresholds
                let p = insert_row(pack, schema, "ceo_thresholds_tables", stem, &row![
                    "key" => &career_key,
                    "ceo" => &career_key,
                    "point_threshold_to_activate" => "1",
                    "point_theshold_to_destroy" => "0",
                    "starting_points" => "1",
                    "max_points" => "1",
                    "resets_to_starting_points_when_deactivated" => "false",
                ])?;
                if !added_paths.contains(&p) { added_paths.push(p); }

                // ceo_nodes
                let p = insert_row(pack, schema, "ceo_nodes_tables", stem, &row![
                    "key" => &career_key,
                    "ceo_effect_list" => &effect_list_key,
                    "title" => "placeholder",
                    "description" => "placeholder",
                    "icon_path" => "",
                    "opinion_topic_modifier" => "",
                    "point_change_per_turn_if_active" => "0",
                ])?;
                if !added_paths.contains(&p) { added_paths.push(p); }

                // ceo_threshold_nodes
                let p = insert_row(pack, schema, "ceo_threshold_nodes_tables", stem, &row![
                    "ceo_threshold" => &career_key,
                    "ceo_node" => &career_key,
                    "points_threshold_to_activate_node" => "1",
                    "can_downgrade_to_previous_node" => "false",
                    "auto_id" => auto_id("ceo_threshold_nodes", &format!("{career_key}|{career_key}")),
                ])?;
                if !added_paths.contains(&p) { added_paths.push(p); }

                // ceo_effect_list_to_effects
                for (effect, val) in &[
                    ("3k_main_character_wealth", "2"),
                    ("3k_main_effect_character_num_lives", "1"),
                ] {
                    let p = insert_row(pack, schema, "ceo_effect_list_to_effects_tables", stem, &row![
                        "effect_list" => &effect_list_key,
                        "effect" => *effect,
                        "value" => *val,
                        "effect_scope" => "character_to_character_own",
                        "optional_only_in_game_mode" => "",
                        "auto_id" => auto_id("ceo_effect_list_to_effects", &format!("{effect_list_key}|{effect}")),
                    ])?;
                    if !added_paths.contains(&p) { added_paths.push(p); }
                }

                // Traits
                for (trait_uuid, trait_key) in &entry.traits {
                    let p = insert_row(pack, schema, "ceo_initial_data_active_ceos_tables", stem, &row![
                        "initial_data_stage" => &stage1_key,
                        "active_ceo" => trait_key.as_str(),
                        "starting_points_delta" => "0",
                        "auto_id" => auto_id("ceo_initial_data_active_ceos", &format!("{stage1_key}|{trait_key}|{trait_uuid}")),
                    ])?;
                    if !added_paths.contains(&p) { added_paths.push(p); }
                }

                // Loc entries for title
                loc_entries.push((format!("ceo_nodes_title_3k_main_ceo_career_historical_{n}"), "PLACEHOLDER".into()));
                loc_entries.push((format!("ceo_nodes_description_3k_main_ceo_career_historical_{n}"), "PLACEHOLDER".into()));
            }
        }

        // ── Write loc file ──────────────────────────────────────
        if !loc_entries.is_empty() {
            let loc_pairs: Vec<(&str, &str)> = loc_entries.iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect();
            let p = insert_loc_entries(pack, &loc_path, &loc_pairs)?;
            if !added_paths.contains(&p) { added_paths.push(p); }
        }

        Ok(added_paths)
}

/// Fetch all trait CEOs from the Assembly Kit data, resolving display names via loc.
pub fn get_trait_ceos(deps: &rpfm_extensions::dependencies::Dependencies) -> Vec<(String, String)> {
    let ak_tables = deps.asskit_only_db_tables();
    let mut trait_ceos: Vec<(String, String)> = Vec::new();

    let trait_categories: HashSet<&str> = [
        "3k_main_ceo_category_traits_personality",
        "3k_main_ceo_category_traits_physical",
    ].iter().copied().collect();

    info!("GetTraitCeos: AK tables available: {}", ak_tables.len());

    // Helper to read rows from an AK table by name, extracting two columns.
    fn ak_lookup_pairs(ak_tables: &HashMap<String, DB>, table_name: &str, col_a: &str, col_b: &str) -> Vec<(String, String)> {
        let mut result = Vec::new();
        if let Some(db) = ak_tables.get(table_name) {
            let fields = db.definition().fields_processed();
            let a_idx = fields.iter().position(|f| f.name() == col_a);
            let b_idx = fields.iter().position(|f| f.name() == col_b);
            if let (Some(ai), Some(bi)) = (a_idx, b_idx) {
                for row in db.data().iter() {
                    result.push((row[ai].data_to_string().to_string(), row[bi].data_to_string().to_string()));
                }
            }
        }
        result
    }

    // Step 1: Get all CEO keys that belong to trait categories from AK.
    if let Some(ceos_db) = ak_tables.get("ceos_tables") {
        let fields = ceos_db.definition().fields_processed();
        let key_idx = fields.iter().position(|f| f.name() == "key");
        let cat_idx = fields.iter().position(|f| f.name() == "category");

        if let (Some(ki), Some(ci)) = (key_idx, cat_idx) {
            for row in ceos_db.data().iter() {
                let category = row[ci].data_to_string();
                if trait_categories.contains(&*category) {
                    let ceo_key = row[ki].data_to_string().to_string();
                    trait_ceos.push((ceo_key, String::new()));
                }
            }
        }
    }

    info!("GetTraitCeos: found {} trait CEOs from AK ceos_tables", trait_ceos.len());

    // Step 2: Walk ceos -> ceo_thresholds -> ceo_threshold_nodes -> ceo_nodes for display names.
    let ceo_to_threshold: HashMap<String, String> =
        ak_lookup_pairs(ak_tables, "ceo_thresholds_tables", "ceo", "key")
            .into_iter().collect();

    let threshold_to_node: HashMap<String, String> =
        ak_lookup_pairs(ak_tables, "ceo_threshold_nodes_tables", "ceo_threshold", "ceo_node")
            .into_iter().collect();

    let node_to_title: HashMap<String, String> =
        ak_lookup_pairs(ak_tables, "ceo_nodes_tables", "key", "title")
            .into_iter().collect();

    // Build loc lookup from dependencies (game files have the loc data).
    let mut loc_lookup: HashMap<String, String> = HashMap::new();
    if let Ok(loc_files) = deps.loc_data(true, true) {
        for rfile in &loc_files {
            if let Ok(RFileDecoded::Loc(loc)) = rfile.decoded() {
                for row in loc.data().iter() {
                    if row.len() >= 2 {
                        let loc_key = row[0].data_to_string().to_string();
                        let loc_val = row[1].data_to_string().to_string();
                        loc_lookup.insert(loc_key, loc_val);
                    }
                }
            }
        }
    }

    info!("GetTraitCeos: loc_lookup has {} entries, node_to_title has {} entries", loc_lookup.len(), node_to_title.len());

    // Step 3: Resolve display names.
    for (ceo_key, display_name) in &mut trait_ceos {
        // Chain: ceo_key -> threshold -> node -> title (loc key) -> loc text
        let resolved = ceo_to_threshold.get(ceo_key.as_str())
            .and_then(|thresh| threshold_to_node.get(thresh))
            .and_then(|node| {
                // Try the title field value as a direct loc key
                node_to_title.get(node).and_then(|title_key| {
                    loc_lookup.get(title_key)
                        .or_else(|| {
                            // Try constructing "ceo_nodes_title_{node_key}"
                            let constructed = format!("ceo_nodes_title_{}", node);
                            loc_lookup.get(&constructed)
                        })
                })
            });

        if let Some(name) = resolved {
            *display_name = name.clone();
        } else {
            // Fallback: humanize the key
            *display_name = ceo_key
                .replace("3k_main_ceo_trait_", "")
                .replace("3k_dlc", "")
                .replace("3k_ytr_ceo_trait_", "")
                .replace('_', " ");
        }
    }

    // Sort by display name for the UI.
    trait_ceos.sort_by(|a, b| a.1.cmp(&b.1));

    info!("GetTraitCeos: returning {} traits", trait_ceos.len());

    trait_ceos
}

/// Import ceo_data.ccd into the pack after BOB has run.
pub fn build_ceo_post(pack: &mut Pack, akit_path: &str) -> Result<Vec<ContainerPath>> {
    let ceo_ccd_path = PathBuf::from(akit_path)
        .join(r"working_data\campaigns\ceo_data.ccd");

    if !ceo_ccd_path.exists() {
        return Err(anyhow!("ceo_data.ccd not found. Make sure BOB ran successfully."));
    }

    let raw_bytes = std::fs::read(&ceo_ccd_path)
        .map_err(|e| anyhow!("Failed to read ceo_data.ccd: {e}"))?;

    let mut rfile = RFile::new_from_vec(&raw_bytes, FileType::Unknown, 0, "campaigns/ceo_data.ccd");
    let _ = rfile.guess_file_type();
    match pack.insert(rfile) {
        Ok(Some(path)) => Ok(vec![path]),
        Ok(None) => Ok(vec![]),
        Err(e) => Err(anyhow!("{}", e)),
    }
}
