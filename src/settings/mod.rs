// In this module should be everything related to the settings stuff.
extern crate serde_json;
extern crate failure;

use std::path::PathBuf;
use std::fs::File;
use std::io::Write;

use self::failure::Error;

/// `GameInfo`: This struct holds all the info needed for a game to be "supported" by RPFM features.
/// It's stores the following data:
/// - `display_name`: This is the name it'll show up in the UI. For example, in a dropdown.
/// - `folder_name`: This name is the name used for any internal operation. For example, for the MyMod stuff.
#[derive(Clone, Debug)]
pub struct GameInfo {
    pub display_name: String,
    pub folder_name: String,
}

/// This struct hold every setting of the program, and it's the one that we are going to serialize.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub paths: Paths,
    pub default_game: String,
    pub prefer_dark_theme: bool,
    pub font: String,
}

/// This struct should hold any path we need to store in the settings.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Paths {
    pub my_mods_base_path: Option<PathBuf>,
    pub warhammer_2: Option<PathBuf>,
    pub warhammer: Option<PathBuf>,
    pub attila: Option<PathBuf>,
    pub rome_2: Option<PathBuf>,
}

/// This struct holds the data needed for the Game Selected.
#[derive(Clone, Debug)]
pub struct GameSelected {
    pub game: String,
    pub game_path: Option<PathBuf>,
    pub game_data_path: Option<PathBuf>,
}

/// Implementation of `GameInfo`.
impl GameInfo {

    /// This function creates a `GameInfo` for every supported game, and returns them in `Vec<GameInfo>`.
    /// NOTE: This vector should NEVER be reordered.
    pub fn new() -> Vec<Self> {

        // List of supported games. To add one, just copy/paste one of the already supported, change
        // the fields for the ones of the new game, and push it to this vector.
        let mut supported_games = vec![];

        // Warhammer 2
        let game_info = GameInfo {
            display_name: "Warhammer 2".to_owned(),
            folder_name: "warhammer_2".to_owned(),
        };

        supported_games.push(game_info);

        // Warhammer
        let game_info = GameInfo {
            display_name: "Warhammer".to_owned(),
            folder_name: "warhammer".to_owned(),
        };

        supported_games.push(game_info);
/*
        // Attila
        let game_info = GameInfo {
            display_name: "Attila".to_owned(),
            folder_name: "attila".to_owned(),
        };

        supported_games.push(game_info);

        // Rome 2
        let game_info = GameInfo {
            display_name: "Rome 2".to_owned(),
            folder_name: "rome_2".to_owned(),
        };

        supported_games.push(game_info);
*/
        // Return the list.
        supported_games
    }
}

/// Implementation of `Settings`.
impl Settings {

    /// This function creates a new settings file with default values and loads it into memory.
    /// Should be run if no settings file has been found at the start of the program.
    pub fn new() -> Settings {
        Settings {
            paths: Paths::new(),
            default_game: "warhammer_2".to_owned(),
            prefer_dark_theme: false,
            font: "Segoe UI 10".to_owned()
        }
    }

    /// This function takes a settings.json file and reads it into a "Settings" object.
    pub fn load() -> Result<Settings, Error> {
        let settings_file = File::open("settings.json")?;
        let settings = serde_json::from_reader(settings_file)?;
        Ok(settings)
    }

    /// This function takes the Settings object and saves it into a settings.json file.
    pub fn save(&self) -> Result<(), Error> {
        let settings_json = serde_json::to_string_pretty(self);
        match File::create(PathBuf::from("settings.json")) {
            Ok(mut file) => {
                match file.write_all(settings_json.unwrap().as_bytes()) {
                    Ok(_) => Ok(()),
                    Err(_) => Err(format_err!("Error while trying to write the \"settings.json\" file.")),
                }
            },
            Err(_) => Err(format_err!("Error while trying prepare the \"settings.json\" file to be written."))
        }
    }
}

/// Implementation of `Paths`.
impl Paths {

    /// This function creates a set of empty paths. Just for the initial creation of the settings file.
    pub fn new() -> Paths {
        Paths {
            my_mods_base_path: None,
            warhammer_2: None,
            warhammer: None,
            attila: None,
            rome_2: None,
        }
    }
}

/// Implementation of `GameSelected`.
impl GameSelected {

    /// This functions returns a GameSelected populated with it's default values..
    pub fn new(settings: &Settings) -> GameSelected {

        let mut game_selected = GameSelected {
            game: "warhammer_2".to_owned(),
            game_path: None,
            game_data_path: None
        };

        match &*settings.default_game {
            "warhammer_2" => {
                game_selected.game = "warhammer_2".to_owned();
                game_selected.game_path = settings.paths.warhammer_2.clone();
                let mut data_path = game_selected.game_path.clone().unwrap_or(PathBuf::from("error"));
                data_path.push("data");
                game_selected.game_data_path = Some(data_path);
            },

            "warhammer" => {
                game_selected.game = "warhammer".to_owned();
                game_selected.game_path = settings.paths.warhammer.clone();
                let mut data_path = game_selected.game_path.clone().unwrap_or(PathBuf::from("error"));
                data_path.push("data");
                game_selected.game_data_path = Some(data_path);
            },

            "attila" => {
                game_selected.game = "attila".to_owned();
                game_selected.game_path = settings.paths.attila.clone();
                let mut data_path = game_selected.game_path.clone().unwrap_or(PathBuf::from("error"));
                data_path.push("data");
                game_selected.game_data_path = Some(data_path);
            },

            "rome_2" => {
                game_selected.game = "rome_2".to_owned();
                game_selected.game_path = settings.paths.rome_2.clone();
                let mut data_path = game_selected.game_path.clone().unwrap_or(PathBuf::from("error"));
                data_path.push("data");
                game_selected.game_data_path = Some(data_path);
            },

            // This should be an error somewhere in the code.
            _ => {
                game_selected.game_path = None;
            },
        }

        game_selected
    }

    /// This functions just changes the values in `GameSelected`.
    pub fn change_game_selected(&mut self, game: &str, game_path: &Option<PathBuf>) {
        self.game = game.to_owned();
        self.game_path = game_path.clone();

        if let Some(ref game_path) = self.game_path {
            let mut data_path = game_path.clone();
            data_path.push("data");
            self.game_data_path = Some(data_path);
        }
    }
}
