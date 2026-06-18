//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Typed constants for all settings keys used across the application.
//!
//! Using these constants instead of raw string literals prevents typos
//! and provides a single source of truth for settings key names.

use serde::{Deserialize, Serialize};

use std::collections::HashMap;

// Path settings.
pub const MYMOD_BASE_PATH: &str = "mymods_base_path";
pub const SECONDARY_PATH: &str = "secondary_path";
pub const CUSTOM_CONFIG_PATH_KEY: &str = "custom_config_path";

/// Suffix appended to a game key to form the settings key for that game's
/// Assembly Kit path (e.g. `warhammer_3` → `warhammer_3_assembly_kit`).
pub const ASSEMBLY_KIT_SUFFIX: &str = "_assembly_kit";

// General settings.
pub const DEFAULT_GAME: &str = "default_game";
pub const LANGUAGE: &str = "language";

/// UI theme preference. One of [`THEME_OS`], [`THEME_LIGHT`] or [`THEME_DARK`].
pub const THEME: &str = "theme";

/// [`THEME`] value: follow the OS light/dark preference (the default).
pub const THEME_OS: &str = "os";
/// [`THEME`] value: force the light scheme regardless of the OS preference.
pub const THEME_LIGHT: &str = "light";
/// [`THEME`] value: force the dark scheme regardless of the OS preference.
pub const THEME_DARK: &str = "dark";

pub const UPDATE_CHANNEL: &str = "update_channel";
pub const AUTOSAVE_AMOUNT: &str = "autosave_amount";
pub const AUTOSAVE_INTERVAL: &str = "autosave_interval";
pub const FONT_NAME: &str = "font_name";
pub const FONT_SIZE: &str = "font_size";
pub const ORIGINAL_FONT_NAME: &str = "original_font_name";
pub const ORIGINAL_FONT_SIZE: &str = "original_font_size";
pub const RECENT_FILE_LIST: &str = "recentFileList";

// UI settings.
pub const START_MAXIMIZED: &str = "start_maximized";
pub const ALLOW_EDITING_OF_CA_PACKFILES: &str = "allow_editing_of_ca_packfiles";
pub const CHECK_UPDATES_ON_START: &str = "check_updates_on_start";
pub const CHECK_SCHEMA_UPDATES_ON_START: &str = "check_schema_updates_on_start";
pub const CHECK_LUA_AUTOGEN_UPDATES_ON_START: &str = "check_lua_autogen_updates_on_start";
pub const CHECK_OLD_AK_UPDATES_ON_START: &str = "check_old_ak_updates_on_start";
pub const USE_LAZY_LOADING: &str = "use_lazy_loading";
pub const DISABLE_UUID_REGENERATION_ON_DB_TABLES: &str = "disable_uuid_regeneration_on_db_tables";
pub const PACKFILE_TREEVIEW_RESIZE_TO_FIT: &str = "packfile_treeview_resize_to_fit";
pub const EXPAND_TREEVIEW_WHEN_ADDING_ITEMS: &str = "expand_treeview_when_adding_items";
pub const USE_RIGHT_SIZE_MARKERS: &str = "use_right_size_markers";
pub const DISABLE_FILE_PREVIEWS: &str = "disable_file_previews";
pub const INCLUDE_BASE_FOLDER_ON_ADD_FROM_FOLDER: &str = "include_base_folder_on_add_from_folder";
pub const DELETE_EMPTY_FOLDERS_ON_DELETE: &str = "delete_empty_folders_on_delete";
pub const AUTOSAVE_FOLDER_SIZE_WARNING_TRIGGERED: &str = "autosave_folder_size_warning_triggered";
pub const IGNORE_GAME_FILES_IN_AK: &str = "ignore_game_files_in_ak";
pub const ENABLE_MULTIFOLDER_FILEPICKER: &str = "enable_multifolder_filepicker";
pub const ENABLE_PACK_CONTENTS_DRAG_AND_DROP: &str = "enable_pack_contents_drag_and_drop";
pub const CLEAN_UI: &str = "clean_ui";
pub const SINGLE_PACK_MODE: &str = "single_pack_mode";

// Hidden/migration settings.
pub const IMPORT_FROM_QT: &str = "import_from_qt";

// Table settings.
pub const ADJUST_COLUMNS_TO_CONTENT: &str = "adjust_columns_to_content";
pub const EXTEND_LAST_COLUMN_ON_TABLES: &str = "extend_last_column_on_tables";
pub const DISABLE_COMBOS_ON_TABLES: &str = "disable_combos_on_tables";
pub const TIGHT_TABLE_MODE: &str = "tight_table_mode";
pub const TABLE_RESIZE_ON_EDIT: &str = "table_resize_on_edit";
pub const TABLES_USE_OLD_COLUMN_ORDER: &str = "tables_use_old_column_order";
pub const TABLES_USE_OLD_COLUMN_ORDER_FOR_TSV: &str = "tables_use_old_column_order_for_tsv";
pub const ENABLE_LOOKUPS: &str = "enable_lookups";
pub const ENABLE_ICONS: &str = "enable_icons";
pub const ENABLE_DIFF_MARKERS: &str = "enable_diff_markers";
pub const HIDE_UNUSED_COLUMNS: &str = "hide_unused_columns";
pub const SHOW_TABLE_TOOLBAR: &str = "show_table_toolbar";

// Debug settings.
pub const CHECK_FOR_MISSING_TABLE_DEFINITIONS: &str = "check_for_missing_table_definitions";
pub const ENABLE_DEBUG_MENU: &str = "enable_debug_menu";
pub const ENABLE_UNIT_EDITOR: &str = "enable_unit_editor";
pub const ENABLE_ESF_EDITOR: &str = "enable_esf_editor";
pub const USE_DEBUG_VIEW_UNIT_VARIANT: &str = "use_debug_view_unit_variant";
pub const ENABLE_RENDERER: &str = "enable_renderer";

// Diagnostics settings.
pub const DIAGNOSTICS_TRIGGER_ON_OPEN: &str = "diagnostics_trigger_on_open";
pub const DIAGNOSTICS_TRIGGER_ON_TABLE_EDIT: &str = "diagnostics_trigger_on_table_edit";

// Telemetry settings.
pub const ENABLE_USAGE_TELEMETRY: &str = "enable_usage_telemetry";
pub const ENABLE_CRASH_REPORTS: &str = "enable_crash_reports";

/// Anonymous per-install id for telemetry.
pub const ANONYMOUS_TELEMETRY_ID: &str = "anonymous_telemetry_id";

// AI settings.
//
// The AI translator talks to any service that exposes the OpenAI
// chat-completions wire format (OpenAI, Anthropic OpenAI-compat,
// Gemini OpenAI-compat, OpenRouter, Ollama, vLLM, LM Studio, ...).
// The user provides the full endpoint URL, an API key sent as a
// `Bearer` token, and the model identifier to target.
pub const AI_API_URL: &str = "ai_api_url";
pub const AI_API_KEY: &str = "ai_api_key";
pub const AI_MODEL: &str = "ai_model";
pub const DEEPL_API_KEY: &str = "deepl_api_key";

// Optimizer settings.
pub const PACK_REMOVE_ITM_FILES: &str = "pack_remove_itm_files";
pub const PACK_APPLY_COMPRESSION: &str = "pack_apply_compression";
pub const PACK_REMOVE_DUPLICATED_FILES: &str = "pack_remove_duplicated_files";
pub const DB_IMPORT_DATACORES_INTO_TWAD_KEY_DELETES: &str = "db_import_datacores_into_twad_key_deletes";
pub const DB_OPTIMIZE_DATACORED_TABLES: &str = "db_optimize_datacored_tables";
pub const TABLE_REMOVE_DUPLICATED_ENTRIES: &str = "table_remove_duplicated_entries";
pub const TABLE_REMOVE_ITM_ENTRIES: &str = "table_remove_itm_entries";
pub const TABLE_REMOVE_ITNR_ENTRIES: &str = "table_remove_itnr_entries";
pub const TABLE_REMOVE_EMPTY_FILE: &str = "table_remove_empty_file";
pub const TEXT_REMOVE_UNUSED_XML_MAP_FOLDERS: &str = "text_remove_unused_xml_map_folders";
pub const TEXT_REMOVE_UNUSED_XML_PREFAB_FOLDER: &str = "text_remove_unused_xml_prefab_folder";
pub const TEXT_REMOVE_AGF_FILES: &str = "text_remove_agf_files";
pub const TEXT_REMOVE_MODEL_STATISTICS_FILES: &str = "text_remove_model_statistics_files";
pub const PTS_REMOVE_UNUSED_ART_SETS: &str = "pts_remove_unused_art_sets";
pub const PTS_REMOVE_UNUSED_VARIANTS: &str = "pts_remove_unused_variants";
pub const PTS_REMOVE_EMPTY_MASKS: &str = "pts_remove_empty_masks";
pub const PTS_REMOVE_EMPTY_FILE: &str = "pts_remove_empty_file";

// Internal/transient keys.
pub const FACTORY_RESET: &str = "factoryReset";
pub const MOVE_CHECKBOX_STATUS: &str = "move_checkbox_status";
pub const GLOBAL_SEARCH_SOURCES_STATUS: &str = "global_search_sources_status";
pub const GLOBAL_SEARCH_FILES_STATUS: &str = "global_search_files_status";
pub const GEOMETRY: &str = "geometry";
pub const WINDOW_STATE: &str = "windowState";
pub const ORIGINAL_GEOMETRY: &str = "originalGeometry";
pub const ORIGINAL_WINDOW_STATE: &str = "originalWindowState";

// QSettings colour keys (used by both Rust and C++ sides).
pub const COLOUR_LIGHT_TABLE_ADDED: &str = "colour_light_table_added";
pub const COLOUR_LIGHT_TABLE_MODIFIED: &str = "colour_light_table_modified";
pub const COLOUR_LIGHT_DIAGNOSTIC_ERROR: &str = "colour_light_diagnostic_error";
pub const COLOUR_LIGHT_DIAGNOSTIC_WARNING: &str = "colour_light_diagnostic_warning";
pub const COLOUR_LIGHT_DIAGNOSTIC_INFO: &str = "colour_light_diagnostic_info";
pub const COLOUR_DARK_TABLE_ADDED: &str = "colour_dark_table_added";
pub const COLOUR_DARK_TABLE_MODIFIED: &str = "colour_dark_table_modified";
pub const COLOUR_DARK_DIAGNOSTIC_ERROR: &str = "colour_dark_diagnostic_error";
pub const COLOUR_DARK_DIAGNOSTIC_WARNING: &str = "colour_dark_diagnostic_warning";
pub const COLOUR_DARK_DIAGNOSTIC_INFO: &str = "colour_dark_diagnostic_info";

/// A snapshot of all typed settings for batch transfer between server and UI.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct SettingsSnapshot {
    pub bool: HashMap<String, bool>,
    pub i32: HashMap<String, i32>,
    pub f32: HashMap<String, f32>,
    pub string: HashMap<String, String>,
    pub raw_data: HashMap<String, Vec<u8>>,
    pub vec_string: HashMap<String, Vec<String>>,
}
