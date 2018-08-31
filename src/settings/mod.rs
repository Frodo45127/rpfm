// In this module should be everything related to the settings stuff.
extern crate serde_json;

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::fs::File;
use std::io::Write;
use std::io::{BufReader, BufWriter};

use error::Result;
use RPFM_PATH;
use SUPPORTED_GAMES;

pub mod shortcuts;

const SETTINGS_FILE: &str = "settings.json";

/// `GameInfo`: This struct holds all the info needed for a game to be "supported" by RPFM features.
/// It's stores the following data:
/// - `display_name`: This is the name it'll show up in the UI. For example, in a dropdown (Warhammer 2).
/// - `id`: This is the ID used at the start of every PackFile for that game. (PFH5)
/// - `schema`: This is the name of the schema file used for the game. (wh2.json)
/// - `db_pack`: This is the PackFile from where we load the data for db references. Since 1.0, we use data.pack or equivalent for this.
/// - `loc_pack`: This is the PackFile from where we load the data for loc special stuff. This should be the one for english. For other languages, we'll have to search it.
/// - `supports_editing`: True if we can save PackFiles for this game. False if we cannot (Arena). This also affect if we can use this game for "MyMod" stuff.
#[derive(Clone, Debug)]
pub struct GameInfo {
    pub display_name: String,
    pub id: String,
    pub schema: String,
    pub db_pack: String,
    pub loc_pack: String,
    pub supports_editing: bool,
}

/// This struct hold every setting of the program, and it's the one that we are going to serialize.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub paths: BTreeMap<String, Option<PathBuf>>,
    pub settings_string: BTreeMap<String, String>,
    pub settings_bool: BTreeMap<String, bool>,
}

/// This struct holds the data needed for the Game Selected.
/// NOTE: `game` is in this format: `warhammer_2`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameSelected {
    pub game: String,
    pub game_path: Option<PathBuf>,
    pub game_data_path: Option<PathBuf>,
    pub game_dependency_packfile_path: Option<PathBuf>,
}

/// Implementation of `Settings`.
impl Settings {

    /// This function creates a new settings file with default values and loads it into memory.
    /// Should be run if no settings file has been found at the start of the program. It requires
    /// the list of supported games, so it can store the game paths properly.
    pub fn new() -> Self {

        // Create the maps to hold the settings.
        let mut paths = BTreeMap::new();
        let mut settings_string = BTreeMap::new();
        let mut settings_bool = BTreeMap::new();

        // Populate the maps with the default shortcuts. New settings MUST BE ADDED HERE.
        paths.insert("mymods_base_path".to_owned(), None);
        
        for (folder_name, _) in SUPPORTED_GAMES.iter() {
            paths.insert(folder_name.to_string(), None);
        }

        settings_string.insert("default_game".to_owned(), "warhammer_2".to_owned());

        settings_bool.insert("adjust_columns_to_content".to_owned(), true);
        settings_bool.insert("allow_editing_of_ca_packfiles".to_owned(), false);
        settings_bool.insert("check_updates_on_start".to_owned(), true);
        settings_bool.insert("check_schema_updates_on_start".to_owned(), true);
        settings_bool.insert("use_pfm_extracting_behavior".to_owned(), false);

        // Return it.
        Self {
            paths,
            settings_string,
            settings_bool,
        }
    }

    /// This function takes a settings.json file and reads it into a "Settings" object.
    pub fn load() -> Result<Self> {

        let path = RPFM_PATH.to_path_buf().join(PathBuf::from(SETTINGS_FILE));
        let file = BufReader::new(File::open(path)?);

        let mut settings: Self = serde_json::from_reader(file)?;

        // Add/Remove settings missing/no-longer-needed for keeping it update friendly. First, remove the outdated ones, then add the new ones.
        let defaults = Self::new();

        {          
            let mut keys_to_delete = vec![];
            for (key, _) in settings.paths.clone() { if let None = defaults.paths.get(&*key) { keys_to_delete.push(key); } }
            for key in &keys_to_delete { settings.paths.remove(key); }

            let mut keys_to_delete = vec![];
            for (key, _) in settings.settings_string.clone() { if let None = defaults.settings_string.get(&*key) { keys_to_delete.push(key); } }
            for key in &keys_to_delete { settings.settings_string.remove(key); }

            let mut keys_to_delete = vec![];
            for (key, _) in settings.settings_bool.clone() { if let None = defaults.settings_bool.get(&*key) { keys_to_delete.push(key); } }
            for key in &keys_to_delete { settings.settings_bool.remove(key); }
        }

        {          
            for (key, value) in defaults.paths { if let None = settings.paths.get(&*key) { settings.paths.insert(key, value);  } }
            for (key, value) in defaults.settings_string { if let None = settings.settings_string.get(&*key) { settings.settings_string.insert(key, value);  } }
            for (key, value) in defaults.settings_bool { if let None = settings.settings_bool.get(&*key) { settings.settings_bool.insert(key, value);  } }
        }

        Ok(settings)
    }

    /// This function takes the Settings object and saves it into a settings.json file.
    pub fn save(&self) -> Result<()> {

        // Try to open the settings file.
        let path = RPFM_PATH.to_path_buf().join(PathBuf::from(SETTINGS_FILE));
        let mut file = BufWriter::new(File::create(path)?);

        // Try to save the file, and return the result.
        let shortcuts = serde_json::to_string_pretty(self);
        file.write_all(shortcuts.unwrap().as_bytes())?;

        // Return success.
        Ok(())
    }
}

/// Implementation of `GameSelected`.
impl GameSelected {

    /// This functions returns a `GameSelected` populated with his default values.
    pub fn new(settings: &Settings) -> Self {

        // Get the stuff we need from the settings and the supported games list.
        let game = settings.settings_string.get("default_game").unwrap().to_owned();
        let game_path = settings.paths.get(&game).unwrap().clone();

        // The data path may be not configured, so we check if it exists in the settings, or not.
        let game_data_path = match game_path {
            Some(ref game_path) => {
                let mut game_data_path = game_path.to_path_buf();
                game_data_path.push("data");
                Some(game_data_path)
            },
            None => None,
        };

        // Same with the data.pack or equivalent.
        let game_dependency_packfile_path = match game_data_path {
            Some(ref game_data_path) => {
                let mut game_db_pack_path = game_data_path.to_path_buf();
                game_db_pack_path.push(SUPPORTED_GAMES.get(&*game).unwrap().db_pack.to_owned());
                Some(game_db_pack_path)
            },
            None => None,
        };

        // Return the final GameSelected.
        Self {
            game,
            game_path,
            game_data_path,
            game_dependency_packfile_path,
        }
    }

    /// This functions just changes the values in `GameSelected`.
    pub fn change_game_selected(&mut self, game: &str, game_path: &Option<PathBuf>) {
        self.game = game.to_owned();
        self.game_path = game_path.clone();

        // Get the data path, if exists.
        if let Some(ref game_path) = self.game_path {
            let mut data_path = game_path.to_path_buf();
            data_path.push("data");
            self.game_data_path = Some(data_path);
        }

        // Otherwise, set it as None.
        else { self.game_data_path = None }

        // Get the data.pack PackFile's path, if exists.
        if let Some(ref game_data_path) = self.game_data_path {
            let mut db_pack_path = game_data_path.to_path_buf();
            db_pack_path.push(SUPPORTED_GAMES.get(&*self.game).unwrap().db_pack.to_owned());
            self.game_dependency_packfile_path = Some(db_pack_path);
        }

        // Otherwise, set it as None.
        else { self.game_dependency_packfile_path = None }
    }
}
