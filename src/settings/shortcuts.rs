// In this module should be everything related to the settings stuff.
extern crate serde_json;

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::fs::File;
use std::io::Write;
use std::io::{BufReader, BufWriter};

use RPFM_PATH;
use error::Result;

const SHORTCUTS_FILE: &str = "shortcuts.json";

/// This struct hold every shortcut of the program, separated by "Sections". Each section corresponds to a TreeView in the UI.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Shortcuts {
    pub menu_bar_packfile: BTreeMap<String, String>,
    pub menu_bar_about: BTreeMap<String, String>,
    pub tree_view: BTreeMap<String, String>,
    pub pack_files_list: BTreeMap<String, String>,
    pub packed_files_db: BTreeMap<String, String>,
    pub packed_files_loc: BTreeMap<String, String>,
    pub db_decoder_fields: BTreeMap<String, String>,
    pub db_decoder_definitions: BTreeMap<String, String>,
}

/// Implementation of `Shortcuts`.
impl Shortcuts {

    /// This function creates a new default set of Shortcuts.
    pub fn new() -> Self {

        // Create the maps to hold the shortcuts.
        let mut menu_bar_packfile = BTreeMap::new();
        let mut menu_bar_about = BTreeMap::new();
        let mut tree_view = BTreeMap::new();
        let mut pack_files_list = BTreeMap::new();
        let mut packed_files_db = BTreeMap::new(); 
        let mut packed_files_loc = BTreeMap::new();
        let mut db_decoder_fields = BTreeMap::new();
        let mut db_decoder_definitions = BTreeMap::new();

        // Populate the maps with the default shortcuts. New shortcuts MUST BE ADDED HERE.
        menu_bar_packfile.insert("new_packfile".to_owned(), "Ctrl+N".to_owned());
        menu_bar_packfile.insert("open_packfile".to_owned(), "Ctrl+O".to_owned());
        menu_bar_packfile.insert("save_packfile".to_owned(), "Ctrl+S".to_owned());
        menu_bar_packfile.insert("save_packfile_as".to_owned(), "Ctrl+Shift+S".to_owned());
        menu_bar_packfile.insert("load_all_ca_packfiles".to_owned(), "Ctrl+G".to_owned());
        menu_bar_packfile.insert("preferences".to_owned(), "Ctrl+P".to_owned());
        menu_bar_packfile.insert("quit".to_owned(), "Ctrl+Q".to_owned());

        menu_bar_about.insert("about_qt".to_owned(), "Ctrl+Alt+H".to_owned());
        menu_bar_about.insert("about_rpfm".to_owned(), "Ctrl+Shift+H".to_owned());
        menu_bar_about.insert("open_manual".to_owned(), "Ctrl+H".to_owned());
        menu_bar_about.insert("check_updates".to_owned(), "Ctrl+U".to_owned());
        menu_bar_about.insert("check_schema_updates".to_owned(), "Ctrl+Shift+U".to_owned());

        tree_view.insert("add_file".to_owned(), "Ctrl+A".to_owned());
        tree_view.insert("add_folder".to_owned(), "Ctrl+Shift+A".to_owned());
        tree_view.insert("add_from_packfile".to_owned(), "Ctrl+Alt+A".to_owned());
        tree_view.insert("create_folder".to_owned(), "Ctrl+F".to_owned());
        tree_view.insert("create_db".to_owned(), "Ctrl+D".to_owned());
        tree_view.insert("create_loc".to_owned(), "Ctrl+L".to_owned());
        tree_view.insert("create_text".to_owned(), "Ctrl+T".to_owned());
        tree_view.insert("mass_import_tsv".to_owned(), "Ctrl+.".to_owned());
        tree_view.insert("mass_export_tsv".to_owned(), "Ctrl+,".to_owned());
        tree_view.insert("delete".to_owned(), "Del".to_owned());
        tree_view.insert("extract".to_owned(), "Ctrl+E".to_owned());
        tree_view.insert("rename_current".to_owned(), "Ctrl+R".to_owned());
        tree_view.insert("apply_prefix_to_selected".to_owned(), "Ctrl+Shift+R".to_owned());
        tree_view.insert("apply_prefix_to_all".to_owned(), "Ctrl+Alt+R".to_owned());
        tree_view.insert("open_in_decoder".to_owned(), "Ctrl+J".to_owned());
        tree_view.insert("open_packfiles_list".to_owned(), "Ctrl+M".to_owned());
        tree_view.insert("open_with_external_program".to_owned(), "Ctrl+K".to_owned());
        tree_view.insert("open_in_multi_view".to_owned(), "Ctrl+B".to_owned());
        tree_view.insert("global_search".to_owned(), "Ctrl+Shift+F".to_owned());
        tree_view.insert("expand_all".to_owned(), "Ctrl++".to_owned());
        tree_view.insert("collapse_all".to_owned(), "Ctrl+-".to_owned());

        pack_files_list.insert("add_row".to_owned(), "Ctrl+Shift+A".to_owned());
        pack_files_list.insert("insert_row".to_owned(), "Ctrl+I".to_owned());
        pack_files_list.insert("delete_row".to_owned(), "Ctrl+Del".to_owned());
        pack_files_list.insert("copy".to_owned(), "Ctrl+C".to_owned());
        pack_files_list.insert("paste".to_owned(), "Ctrl+V".to_owned());
        pack_files_list.insert("paste_as_new_row".to_owned(), "Ctrl+Shift+V".to_owned());
        
        packed_files_db.insert("add_row".to_owned(), "Ctrl+Shift+A".to_owned());
        packed_files_db.insert("insert_row".to_owned(), "Ctrl+I".to_owned());
        packed_files_db.insert("delete_row".to_owned(), "Ctrl+Del".to_owned());
        packed_files_db.insert("clone_row".to_owned(), "Ctrl+D".to_owned());
        packed_files_db.insert("clone_and_append_row".to_owned(), "Ctrl+Shift+D".to_owned());
        packed_files_db.insert("copy".to_owned(), "Ctrl+C".to_owned());
        packed_files_db.insert("copy_as_lua_table".to_owned(), "Ctrl+Shift+C".to_owned());
        packed_files_db.insert("paste_in_selection".to_owned(), "Ctrl+V".to_owned());
        packed_files_db.insert("paste_as_new_row".to_owned(), "Ctrl+Shift+V".to_owned());
        packed_files_db.insert("paste_to_fill_selection".to_owned(), "Ctrl+Alt+V".to_owned());
        packed_files_db.insert("apply_maths_to_selection".to_owned(), "Ctrl+B".to_owned());
        packed_files_db.insert("search".to_owned(), "Ctrl+F".to_owned());
        packed_files_db.insert("import_tsv".to_owned(), "Ctrl+W".to_owned());
        packed_files_db.insert("export_tsv".to_owned(), "Ctrl+E".to_owned());
        packed_files_db.insert("smart_delete".to_owned(), "Del".to_owned());
        packed_files_db.insert("undo".to_owned(), "Ctrl+Z".to_owned());
        packed_files_db.insert("redo".to_owned(), "Ctrl+Shift+Z".to_owned());
        
        packed_files_loc.insert("add_row".to_owned(), "Ctrl+Shift+A".to_owned());
        packed_files_loc.insert("insert_row".to_owned(), "Ctrl+I".to_owned());
        packed_files_loc.insert("delete_row".to_owned(), "Ctrl+Del".to_owned());
        packed_files_loc.insert("clone_row".to_owned(), "Ctrl+D".to_owned());
        packed_files_loc.insert("clone_and_append_row".to_owned(), "Ctrl+Shift+D".to_owned());
        packed_files_loc.insert("copy".to_owned(), "Ctrl+C".to_owned());
        packed_files_loc.insert("paste_in_selection".to_owned(), "Ctrl+V".to_owned());
        packed_files_loc.insert("paste_as_new_row".to_owned(), "Ctrl+Shift+V".to_owned());
        packed_files_loc.insert("paste_to_fill_selection".to_owned(), "Ctrl+Alt+V".to_owned());
        packed_files_loc.insert("search".to_owned(), "Ctrl+F".to_owned());
        packed_files_loc.insert("import_tsv".to_owned(), "Ctrl+W".to_owned());
        packed_files_loc.insert("export_tsv".to_owned(), "Ctrl+E".to_owned());
        packed_files_loc.insert("smart_delete".to_owned(), "Del".to_owned());
        packed_files_loc.insert("undo".to_owned(), "Ctrl+Z".to_owned());
        packed_files_loc.insert("redo".to_owned(), "Ctrl+Shift+Z".to_owned());
        
        db_decoder_fields.insert("move_up".to_owned(), "Ctrl+Up".to_owned());
        db_decoder_fields.insert("move_down".to_owned(), "Ctrl+Down".to_owned());
        db_decoder_fields.insert("delete".to_owned(), "Ctrl+Del".to_owned());

        db_decoder_definitions.insert("load".to_owned(), "Ctrl+L".to_owned());
        db_decoder_definitions.insert("delete".to_owned(), "Ctrl+Del".to_owned());

        // Return it.
        Self {
            menu_bar_packfile,
            menu_bar_about,
            tree_view,
            pack_files_list,
            packed_files_db,
            packed_files_loc,
            db_decoder_fields,
            db_decoder_definitions,
        }
    }

    /// This function takes a shortcuts.json file and reads it into a `Shortcuts` object. It has to receive the RPFM's path.
    pub fn load() -> Result<Self> {

    	// Try to open the shortcuts file.
        let path = RPFM_PATH.to_path_buf().join(PathBuf::from(SHORTCUTS_FILE));
        let file = BufReader::new(File::open(path)?);

        // Try to get the shortcuts. This can fail because the file is changed or damaged, or because there is no file.
        let mut shortcuts: Self = serde_json::from_reader(file)?;

        // Add/Remove shortcuts missing/no-longer-needed for keeping it update friendly. First, remove the outdated ones, then add the new ones.
        let defaults = Self::new();

        {          
            let mut keys_to_delete = vec![];
            for (key, _) in shortcuts.menu_bar_packfile.clone() { if let None = defaults.menu_bar_packfile.get(&*key) { keys_to_delete.push(key); } }
            for key in &keys_to_delete { shortcuts.menu_bar_packfile.remove(key); }

            let mut keys_to_delete = vec![];
            for (key, _) in shortcuts.menu_bar_about.clone() { if let None = defaults.menu_bar_about.get(&*key) { keys_to_delete.push(key); } }
            for key in &keys_to_delete { shortcuts.menu_bar_about.remove(key); }

            let mut keys_to_delete = vec![];
            for (key, _) in shortcuts.tree_view.clone() { if let None = defaults.tree_view.get(&*key) { keys_to_delete.push(key); } }
            for key in &keys_to_delete { shortcuts.tree_view.remove(key); }

            let mut keys_to_delete = vec![];
            for (key, _) in shortcuts.pack_files_list.clone() { if let None = defaults.pack_files_list.get(&*key) { keys_to_delete.push(key); } }
            for key in &keys_to_delete { shortcuts.pack_files_list.remove(key); }

            let mut keys_to_delete = vec![];
            for (key, _) in shortcuts.packed_files_db.clone() { if let None = defaults.packed_files_db.get(&*key) { keys_to_delete.push(key); } }
            for key in &keys_to_delete { shortcuts.packed_files_db.remove(key); }

            let mut keys_to_delete = vec![];
            for (key, _) in shortcuts.packed_files_loc.clone() { if let None = defaults.packed_files_loc.get(&*key) { keys_to_delete.push(key); } }
            for key in &keys_to_delete { shortcuts.packed_files_loc.remove(key); }

            let mut keys_to_delete = vec![];
            for (key, _) in shortcuts.db_decoder_fields.clone() { if let None = defaults.db_decoder_fields.get(&*key) { keys_to_delete.push(key); } }
            for key in &keys_to_delete { shortcuts.db_decoder_fields.remove(key); }

            let mut keys_to_delete = vec![];
            for (key, _) in shortcuts.db_decoder_definitions.clone() { if let None = defaults.db_decoder_definitions.get(&*key) { keys_to_delete.push(key); } }
            for key in &keys_to_delete { shortcuts.db_decoder_definitions.remove(key); }
        }

        {          
            for (key, value) in defaults.menu_bar_packfile { if let None = shortcuts.menu_bar_packfile.get(&*key) { shortcuts.menu_bar_packfile.insert(key, value);  } }
            for (key, value) in defaults.menu_bar_about { if let None = shortcuts.menu_bar_about.get(&*key) { shortcuts.menu_bar_about.insert(key, value);  } }
            for (key, value) in defaults.tree_view { if let None = shortcuts.tree_view.get(&*key) { shortcuts.tree_view.insert(key, value);  } }
            for (key, value) in defaults.pack_files_list { if let None = shortcuts.pack_files_list.get(&*key) { shortcuts.pack_files_list.insert(key, value);  } }
            for (key, value) in defaults.packed_files_db { if let None = shortcuts.packed_files_db.get(&*key) { shortcuts.packed_files_db.insert(key, value);  } }
            for (key, value) in defaults.packed_files_loc { if let None = shortcuts.packed_files_loc.get(&*key) { shortcuts.packed_files_loc.insert(key, value);  } }
            for (key, value) in defaults.db_decoder_fields { if let None = shortcuts.db_decoder_fields.get(&*key) { shortcuts.db_decoder_fields.insert(key, value);  } }
            for (key, value) in defaults.db_decoder_definitions { if let None = shortcuts.db_decoder_definitions.get(&*key) { shortcuts.db_decoder_definitions.insert(key, value);  } }
        }

        // Return the shortcuts.
        Ok(shortcuts)
    }

    /// This function takes the `Shortcuts` object and saves it into a shortcuts.json file.
    pub fn save(&self) -> Result<()> {
       
        // Try to open the shortcuts file.
        let path = RPFM_PATH.to_path_buf().join(PathBuf::from(SHORTCUTS_FILE));
        let mut file = BufWriter::new(File::create(path)?);

        // Try to save the file, and return the result.
        let shortcuts = serde_json::to_string_pretty(self);
        file.write_all(shortcuts.unwrap().as_bytes())?;

        // Return success.
        Ok(())
    }
}
