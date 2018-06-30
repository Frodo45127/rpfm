// In this module should be everything related to the settings stuff.
extern crate serde_json;
extern crate failure;

use std::path::PathBuf;
use std::fs::File;
use std::io::Write;
use std::io::BufReader;

use failure::Error;

/// `GameInfo`: This struct holds all the info needed for a game to be "supported" by RPFM features.
/// It's stores the following data:
/// - `display_name`: This is the name it'll show up in the UI. For example, in a dropdown.
/// - `folder_name`: This name is the name used for any internal operation. For example, for the MyMod stuff.
/// - `id`: This is the ID used at the start of every PackFile for that game.
/// - `dependency_pack`: The name of the "Dependency" PackFile for that game.
#[derive(Clone, Debug)]
pub struct GameInfo {
    pub display_name: String,
    pub folder_name: String,
    pub id: String,
    pub dependency_pack: String,
    pub schema: String,
}

/// This struct hold every setting of the program, and it's the one that we are going to serialize.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub paths: Paths,
    pub default_game: String,
    //pub prefer_dark_theme: bool,
    //pub font: String,
    pub allow_editing_of_ca_packfiles: bool,
    pub check_updates_on_start: bool,
    pub check_schema_updates_on_start: bool,
    pub use_pfm_extracting_behavior: bool,
}

/// This struct should hold any path we need to store in the settings.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Paths {
    pub my_mods_base_path: Option<PathBuf>,
    pub game_paths: Vec<GamePath>,
}

/// This struct should hold the name of a game (folder_name from GameInfo) and his path, if configured.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GamePath {
    pub game: String,
    pub path: Option<PathBuf>,
}

/// This struct holds the data needed for the Game Selected.
/// NOTE: `game` is in this format: `warhammer_2`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameSelected {
    pub game: String,
    pub game_path: Option<PathBuf>,
    pub game_data_path: Option<PathBuf>,
    pub game_dependency_packfile_path: PathBuf,
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
            id: "PFH5".to_owned(),
            dependency_pack: "wh2.pack".to_owned(),
            schema: "schema_wh.json".to_owned(),
        };

        supported_games.push(game_info);

        // Warhammer
        let game_info = GameInfo {
            display_name: "Warhammer".to_owned(),
            folder_name: "warhammer".to_owned(),
            id: "PFH4".to_owned(),
            dependency_pack: "wh.pack".to_owned(),
            schema: "schema_wh.json".to_owned(),
        };

        supported_games.push(game_info);

        // Attila
        let game_info = GameInfo {
            display_name: "Attila".to_owned(),
            folder_name: "attila".to_owned(),
            id: "PFH4".to_owned(),
            dependency_pack: "att.pack".to_owned(),
            schema: "schema_att.json".to_owned(),
        };

        supported_games.push(game_info);
        /*

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
    /// Should be run if no settings file has been found at the start of the program. It requires
    /// the list of supported games, so it can store the game paths properly.
    pub fn new(supported_games: &[GameInfo]) -> Self {
        Self {
            paths: Paths::new(supported_games),
            default_game: "warhammer_2".to_owned(),
            //prefer_dark_theme: false,
            //font: "Segoe UI 9".to_owned(),
            allow_editing_of_ca_packfiles: false,
            check_updates_on_start: true,
            check_schema_updates_on_start: true,
            use_pfm_extracting_behavior: false,
        }
    }

    /// This function takes a settings.json file and reads it into a "Settings" object.
    pub fn load(path: &PathBuf, supported_games: &[GameInfo]) -> Result<Self, Error> {
        let settings_path = path.to_path_buf().join(PathBuf::from("settings.json"));
        let settings_file = BufReader::new(File::open(settings_path)?);
        let mut settings: Self = serde_json::from_reader(settings_file)?;

        // We need to make sure here that we have entries in `game_paths` for every supported game.
        // Otherwise, it'll crash when trying to open the "Preferences" window.
        if settings.paths.game_paths.len() < supported_games.len() {
            for (index, game) in supported_games.iter().enumerate() {
                if settings.paths.game_paths.get(index).is_some() {
                    if settings.paths.game_paths[index].game != game.folder_name {

                        // Something has changed the order of the Games, so we wipe all `GamePath`s
                        // if we hit this, as some of them will be misplaced.
                        settings.paths.game_paths = GamePath::new(supported_games);
                    }
                }
                else {
                    settings.paths.game_paths.push(
                        GamePath {
                            game: game.folder_name.to_owned(),
                            path: None,
                        }
                    );
                }
            }
        }

        Ok(settings)
    }

    /// This function takes the Settings object and saves it into a settings.json file.
    pub fn save(&self, path: &PathBuf) -> Result<(), Error> {
        let mut settings_path = path.clone();
        settings_path.push("settings.json");

        let settings_json = serde_json::to_string_pretty(self);
        match File::create(settings_path) {
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
    pub fn new(supported_games: &[GameInfo]) -> Paths {
        Paths {
            my_mods_base_path: None,
            game_paths: GamePath::new(supported_games)
        }
    }
}

/// Implementation of `GamePath`.
impl GamePath {

    /// This function returns a vector of GamePaths for supported Games.
    pub fn new(supported_games: &[GameInfo]) -> Vec<Self> {

        let mut game_paths = vec![];
        for game in supported_games {
            game_paths.push(
                Self {
                    game: game.folder_name.to_owned(),
                    path: None,
                }
            )
        }

        game_paths
    }
}

/// Implementation of `GameSelected`.
impl GameSelected {

    /// This functions returns a `GameSelected` populated with his default values.
    pub fn new(settings: &Settings, rpfm_path: &PathBuf, supported_games: &[GameInfo]) -> Self {

        let game = settings.default_game.to_owned();
        let game_path = settings.paths.game_paths.iter().filter(|x| x.game == game).map(|x| x.path.clone()).collect::<Option<PathBuf>>();
        let game_data_path = match game_path {
            Some(ref game_path) => {
                let mut game_data_path = game_path.clone();
                game_data_path.push("data");
                Some(game_data_path)
            },
            None => None,
        };

        let mut game_dependency_packfile_path = rpfm_path.to_path_buf();
        game_dependency_packfile_path.push("dependency_packs");
        game_dependency_packfile_path.push(supported_games.iter().filter(|x| x.folder_name == game).map(|x| x.dependency_pack.clone()).collect::<String>());

        Self {
            game,
            game_path,
            game_data_path,
            game_dependency_packfile_path,
        }
    }

    /// This functions just changes the values in `GameSelected`.
    pub fn change_game_selected(&mut self, game: &str, game_path: &Option<PathBuf>, supported_games: &[GameInfo]) {
        self.game = game.to_owned();
        self.game_path = game_path.clone();

        if let Some(ref game_path) = self.game_path {
            let mut data_path = game_path.clone();
            data_path.push("data");
            self.game_data_path = Some(data_path);
        }

        self.game_dependency_packfile_path.pop();
        self.game_dependency_packfile_path.push(supported_games.iter().filter(|x| x.folder_name == game).map(|x| x.dependency_pack.clone()).collect::<String>());
    }
}
