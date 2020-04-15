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
This module contains the code related to the ***Shortcuts*** of every shortcutable action in the Program.

If you ever add a new action to the Program, remember to add it here.
!*/

use ron::de::from_reader;
use ron::ser::{to_string_pretty, PrettyConfig};
use serde_derive::{Serialize, Deserialize};

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::iter::FromIterator;

use rpfm_error::Result;
use rpfm_lib::config::get_config_path;

/// Name of the file which contains the current shortcuts of the program.
const SHORTCUTS_FILE: &str = "shortcuts.ron";

/// List of shortcuts for the `PackFile` Menu.
const SHORTCUTS_MENU_BAR_PACKFILE: [(&str, &str); 7] = [
    ("new_packfile", "Ctrl+N"),
    ("open_packfile", "Ctrl+O"),
    ("save_packfile", "Ctrl+S"),
    ("save_packfile_as", "Ctrl+Shift+S"),
    ("load_all_ca_packfiles", "Ctrl+G"),
    ("preferences", "Ctrl+P"),
    ("quit", "Ctrl+Q"),
];

/// List of shortcuts for the `MyMod` Menu.
const SHORTCUTS_MENU_BAR_MYMOD: [(&str, &str); 4] = [
    ("mymod_new", "Ctrl+Shift+O"),
    ("mymod_delete_selected", "Ctrl+Shift+O"),
    ("mymod_install", "Ctrl+Alt+O"),
    ("mymod_uninstall", "Ctrl+Shift+O"),
];

/// List of shortcuts for the `View` Menu.
const SHORTCUTS_MENU_BAR_VIEW: [(&str, &str); 2] = [
    ("view_toggle_packfile_contents", "Ctrl+Shift+O"),
    ("view_toggle_global_search_panel", "Ctrl+Shift+O"),
];

/// List of shortcuts for the `Game Selected` Menu.
const SHORTCUTS_MENU_BAR_GAME_SELECTED: [(&str, &str); 4] = [
    ("launch_game", "Ctrl+Shift+O"),
    ("open_game_data_folder", "Ctrl+Shift+O"),
    ("open_game_assembly_kit_folder", "Ctrl+Alt+O"),
    ("open_config_folder", "Ctrl+Shift+O"),
];

/// List of shortcuts for the `About` Menu.
const SHORTCUTS_MENU_BAR_ABOUT: [(&str, &str); 6] = [
    ("about_qt", "Ctrl+Alt+H"),
    ("about_rpfm", "Ctrl+Shift+H"),
    ("open_manual", "Ctrl+H"),
    ("support_me_on_patreon", "Ctrl+H"),
    ("check_updates", "Ctrl+U"),
    ("check_schema_updates", "Ctrl+Shift+U"),
];

/// List of shortcuts for the PackFile Contents Contextual Menu.
const SHORTCUTS_PACKFILE_CONTENTS_TREE_VIEW: [(&str, &str); 25] = [
    ("add_file", "Ctrl+A"),
    ("add_folder", "Ctrl+Shift+A"),
    ("add_from_packfile", "Ctrl+Alt+A"),
    ("check_tables", "Ctrl+Shift+I"),
    ("create_folder", "Ctrl+F"),
    ("create_db", "Ctrl+D"),
    ("create_loc", "Ctrl+L"),
    ("create_text", "Ctrl+T"),
    ("create_queek", "Ctrl+T"),
    ("mass_import_tsv", "Ctrl+."),
    ("mass_export_tsv", "Ctrl+,"),
    ("merge_tables", "Ctrl+M"),
    ("update_tables", "Ctrl+M"),
    ("delete", "Del"),
    ("extract", "Ctrl+E"),
    ("rename", "Ctrl+R"),
    ("open_in_decoder", "Ctrl+J"),
    ("open_packfiles_list", "Ctrl+Alt+M"),
    ("open_with_external_program", "Ctrl+K"),
    ("open_containing_folder", "Ctrl+0"),
    ("open_in_multi_view", "Ctrl+B"),
    ("open_notes", "Ctrl+Y"),
    ("global_search", "Ctrl+Shift+F"),
    ("expand_all", "Ctrl++"),
    ("collapse_all", "Ctrl+-"),
];

/// List of shortcuts for the Table PackedFile's Contextual Menu.
const SHORTCUTS_PACKED_FILE_TABLE: [(&str, &str); 20] = [
    ("add_row", "Ctrl+Shift+A"),
    ("insert_row", "Ctrl+I"),
    ("delete_row", "Ctrl+Del"),
    ("clone_row", "Ctrl+D"),
    ("clone_and_append_row", "Ctrl+Shift+D"),
    ("copy", "Ctrl+C"),
    ("copy_as_lua_table", "Ctrl+Shift+C"),
    ("paste", "Ctrl+V"),
    ("paste_as_new_row", "Ctrl+Shift+V"),
    ("revert_selection", "Ctrl+Alt+V"),
    ("apply_maths_to_selection", "Ctrl+B"),
    ("rewrite_selection", "Ctrl+Y"),
    ("selection_invert", "Ctrl+-"),
    ("search", "Ctrl+F"),
    ("sidebar", "Ctrl+A"),
    ("import_tsv", "Ctrl+W"),
    ("export_tsv", "Ctrl+E"),
    ("smart_delete", "Del"),
    ("undo", "Ctrl+Z"),
    ("redo", "Ctrl+Shift+Z"),
];

/// List of shortcuts for the Table Decoder.
const SHORTCUTS_PACKED_FILE_DECODER: [(&str, &str); 4] = [
    ("move_up", "Ctrl+Up"),
    ("move_down", "Ctrl+Down"),
    ("delete", "Ctrl+Del"),
    ("load", "Ctrl+L"),
];

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains every shortcut of the program, separated by sections. Each section corresponds to a Menu/View in the UI.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Shortcuts {
    pub menu_bar_packfile: BTreeMap<String, String>,
    pub menu_bar_mymod: BTreeMap<String, String>,
    pub menu_bar_view: BTreeMap<String, String>,
    pub menu_bar_game_selected: BTreeMap<String, String>,
    pub menu_bar_about: BTreeMap<String, String>,
    pub packfile_contents_tree_view: BTreeMap<String, String>,
    pub packed_file_table: BTreeMap<String, String>,
    pub packed_file_decoder: BTreeMap<String, String>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `Shortcuts`.
impl Shortcuts {

    /// This function creates a new default set of Shortcuts.
    pub fn new() -> Self {
        Self {
            menu_bar_packfile: BTreeMap::from_iter(SHORTCUTS_MENU_BAR_PACKFILE.iter().map(|(x, y)| ((*x).to_string(), (*y).to_string()))),
            menu_bar_mymod: BTreeMap::from_iter(SHORTCUTS_MENU_BAR_MYMOD.iter().map(|(x, y)| ((*x).to_string(), (*y).to_string()))),
            menu_bar_view: BTreeMap::from_iter(SHORTCUTS_MENU_BAR_VIEW.iter().map(|(x, y)| ((*x).to_string(), (*y).to_string()))),
            menu_bar_game_selected: BTreeMap::from_iter(SHORTCUTS_MENU_BAR_GAME_SELECTED.iter().map(|(x, y)| ((*x).to_string(), (*y).to_string()))),
            menu_bar_about: BTreeMap::from_iter(SHORTCUTS_MENU_BAR_ABOUT.iter().map(|(x, y)| ((*x).to_string(), (*y).to_string()))),
            packfile_contents_tree_view: BTreeMap::from_iter(SHORTCUTS_PACKFILE_CONTENTS_TREE_VIEW.iter().map(|(x, y)| ((*x).to_string(), (*y).to_string()))),
            packed_file_table: BTreeMap::from_iter(SHORTCUTS_PACKED_FILE_TABLE.iter().map(|(x, y)| ((*x).to_string(), (*y).to_string()))),
            packed_file_decoder: BTreeMap::from_iter(SHORTCUTS_PACKED_FILE_DECODER.iter().map(|(x, y)| ((*x).to_string(), (*y).to_string()))),
        }
    }

    /// This function creates a `Shortcuts` struct from the configuration file, if exists.
    pub fn load() -> Result<Self> {

        // Try to open the shortcuts file.
        let file_path = get_config_path()?.join(SHORTCUTS_FILE);
        let file = BufReader::new(File::open(file_path)?);

        // Try to get the shortcuts. This can fail because the file is changed or damaged, or because there is no file.
        let mut shortcuts: Self = from_reader(file)?;

        // Add/Remove shortcuts missing/no-longer-needed for keeping it update friendly. First, remove the outdated ones, then add the new ones.
        let defaults = Self::new();

        {
            let mut keys_to_delete = vec![];
            for key in shortcuts.menu_bar_packfile.keys() { if defaults.menu_bar_packfile.get(key).is_none() { keys_to_delete.push(key.clone()); } }
            for key in &keys_to_delete { shortcuts.menu_bar_packfile.remove(key); }

            let mut keys_to_delete = vec![];
            for key in shortcuts.menu_bar_mymod.keys() { if defaults.menu_bar_mymod.get(key).is_none() { keys_to_delete.push(key.clone()); } }
            for key in &keys_to_delete { shortcuts.menu_bar_mymod.remove(key); }

            let mut keys_to_delete = vec![];
            for key in shortcuts.menu_bar_view.keys() { if defaults.menu_bar_view.get(key).is_none() { keys_to_delete.push(key.clone()); } }
            for key in &keys_to_delete { shortcuts.menu_bar_view.remove(key); }

            let mut keys_to_delete = vec![];
            for key in shortcuts.menu_bar_game_selected.keys() { if defaults.menu_bar_game_selected.get(key).is_none() { keys_to_delete.push(key.clone()); } }
            for key in &keys_to_delete { shortcuts.menu_bar_game_selected.remove(key); }

            let mut keys_to_delete = vec![];
            for key in shortcuts.menu_bar_about.keys() { if defaults.menu_bar_about.get(key).is_none() { keys_to_delete.push(key.clone()); } }
            for key in &keys_to_delete { shortcuts.menu_bar_about.remove(key); }

            let mut keys_to_delete = vec![];
            for key in shortcuts.packfile_contents_tree_view.keys() { if defaults.packfile_contents_tree_view.get(key).is_none() { keys_to_delete.push(key.clone()); } }
            for key in &keys_to_delete { shortcuts.packfile_contents_tree_view.remove(key); }

            let mut keys_to_delete = vec![];
            for key in shortcuts.packed_file_table.keys() { if defaults.packed_file_table.get(key).is_none() { keys_to_delete.push(key.clone()); } }
            for key in &keys_to_delete { shortcuts.packed_file_table.remove(key); }

            let mut keys_to_delete = vec![];
            for key in shortcuts.packed_file_decoder.keys() { if defaults.packed_file_decoder.get(key).is_none() { keys_to_delete.push(key.clone()); } }
            for key in &keys_to_delete { shortcuts.packed_file_decoder.remove(key); }
        }

        {
            for (key, value) in defaults.menu_bar_packfile { if shortcuts.menu_bar_packfile.get(&key).is_none() { shortcuts.menu_bar_packfile.insert(key, value);  } }
            for (key, value) in defaults.menu_bar_mymod { if shortcuts.menu_bar_mymod.get(&key).is_none() { shortcuts.menu_bar_mymod.insert(key, value);  } }
            for (key, value) in defaults.menu_bar_view { if shortcuts.menu_bar_view.get(&key).is_none() { shortcuts.menu_bar_view.insert(key, value);  } }
            for (key, value) in defaults.menu_bar_game_selected { if shortcuts.menu_bar_game_selected.get(&key).is_none() { shortcuts.menu_bar_game_selected.insert(key, value);  } }
            for (key, value) in defaults.menu_bar_about { if shortcuts.menu_bar_about.get(&key).is_none() { shortcuts.menu_bar_about.insert(key, value);  } }
            for (key, value) in defaults.packfile_contents_tree_view { if shortcuts.packfile_contents_tree_view.get(&key).is_none() { shortcuts.packfile_contents_tree_view.insert(key, value);  } }
            for (key, value) in defaults.packed_file_table { if shortcuts.packed_file_table.get(&key).is_none() { shortcuts.packed_file_table.insert(key, value);  } }
            for (key, value) in defaults.packed_file_decoder { if shortcuts.packed_file_decoder.get(&key).is_none() { shortcuts.packed_file_decoder.insert(key, value);  } }
        }

        // Return the shortcuts.
        Ok(shortcuts)
    }

    /// This function takes the `Shortcuts` struct and saves it into a shortcuts.json file.
    pub fn save(&self) -> Result<()> {

        // Try to open the shortcuts file.
        let file_path = get_config_path()?.join(SHORTCUTS_FILE);
        let mut file = BufWriter::new(File::create(file_path)?);

        // Try to save the file, and return the result.
        let config = PrettyConfig::default();
        file.write_all(to_string_pretty(&self, config)?.as_bytes())?;

        // Return success.
        Ok(())
    }
}
