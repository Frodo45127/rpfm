//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Methods of the tools: optimizer, maps, startpos, animations, glTF export, MyMods and Lua tests.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use std::collections::BTreeMap;
use std::path::PathBuf;

use rpfm_extensions::optimizer::OptimizerOptions;

use super::{Done, Request};
use super::files::FileRef;
use super::tables::FilesEdited;

/// `tools.optimizer_options`: returns the optimizer options, as the server's settings have them.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct GetOptimizerOptions {}

/// Options of the optimizer, by name, like `table_remove_itm_entries` or `pack_remove_duplicated_files`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct OptimizerOptionValues {

    /// If each optimization is enabled, by name.
    pub options: BTreeMap<String, bool>,
}

/// `tools.optimize`: removes data of an open pack that's identical to the vanilla one, or unneeded. Runs as a job.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct OptimizePack {

    /// Key of the pack.
    pub pack: String,

    /// Optimizations to enable or disable, by name. The rest keep the server's settings.
    /// See [`GetOptimizerOptions`] for the names.
    #[serde(default)]
    pub options: BTreeMap<String, bool>,
}

/// Files changed by an operation.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct FilesChanged {

    /// Paths of the added or edited files.
    pub added: Vec<String>,

    /// Paths of the deleted files.
    pub deleted: Vec<String>,
}

/// `tools.patch_siege_ai`: patches the siege maps of an open pack, so the AI can use them. Warhammer games only.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PatchSiegeAi {

    /// Key of the pack.
    pub pack: String,
}

/// Result of patching siege maps.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SiegeAiPatched {

    /// Description of what was patched.
    pub message: String,

    /// Paths of the files deleted by the patch.
    pub deleted: Vec<String>,
}

/// `tools.pack_map`: adds the tiles and tile maps of a map exported by Terry to an open pack.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PackMap {

    /// Key of the pack.
    pub pack: String,

    /// Paths on disk of the tile maps.
    pub tile_maps: Vec<PathBuf>,

    /// Tiles to add.
    #[serde(default)]
    pub tiles: Vec<MapTile>,
}

/// A tile of a map.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct MapTile {

    /// Path on disk of the tile.
    pub path: PathBuf,

    /// Subfolder of the pack's terrain tiles folder to add it to.
    pub folder: String,
}

/// `tools.generate_missing_locs`: adds empty loc entries for the localised columns of the tables of the open packs that have none.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct GenerateMissingLocs {}

/// `tools.update_anim_ids`: offsets the animation ids of an open pack, like after a game update moves them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct UpdateAnimIds {

    /// Key of the pack.
    pub pack: String,

    /// Only ids from this one are changed.
    pub starting_id: i32,

    /// Amount to add to each id.
    pub offset: i32,
}

/// `tools.anims_by_skeleton`: returns the paths of the animations using a skeleton, in the open packs and the dependencies.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct AnimsBySkeleton {

    /// Name of the skeleton.
    pub skeleton: String,
}

/// Paths of files.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct FilePaths {

    /// The paths, sorted.
    pub paths: Vec<String>,
}

/// `tools.export_gltf`: exports a RigidModel to a glTF file, with its textures.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ExportGltf {

    /// The RigidModel.
    pub file: FileRef,

    /// Path of the glTF file to write.
    pub destination: PathBuf,
}

/// `tools.set_video_format`: changes the format of a ca_vp8 video of an open pack.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct SetVideoFormat {

    /// Key of the pack.
    pub pack: String,

    /// Path of the video.
    pub path: String,

    /// New format: `CaVp8` or `Ivf`.
    pub format: String,
}

/// `tools.live_export`: exports the scripts and UI files of an open pack to the game's data folder, to test them without saving the pack.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct LiveExport {

    /// Key of the pack.
    pub pack: String,
}

/// `tools.init_mymod`: creates the folder of a new MyMod, with editor configs for Lua scripting.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct InitMyMod {

    /// Folder name of the game of the MyMod, like `warhammer_3`.
    pub game_folder: String,

    /// Name of the MyMod.
    pub name: String,

    /// If a Sublime Text project is created.
    #[serde(default)]
    pub sublime_support: bool,

    /// If a VSCode config is created.
    #[serde(default)]
    pub vscode_support: bool,

    /// If set, a git repository is created with this `.gitignore`.
    #[serde(default)]
    pub gitignore: Option<String>,
}

/// A MyMod created.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct MyModCreated {

    /// Path of the pack of the MyMod. It doesn't exist until it's saved.
    pub pack_path: PathBuf,
}

/// `lua.run_tests`: runs Lua tests against the scripts of the game and the open packs, outside of the game. Runs as a job.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RunLuaTests {

    /// Code of the Lua test file.
    pub source: String,

    /// Campaign whose vanilla scripts to load, like `main_warhammer`. If not set, only the script libraries and the mods are loaded.
    #[serde(default)]
    pub campaign: Option<String>,
}

/// Results of Lua tests.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct LuaTestResults {

    /// The report of the tests: passed and failed tests, with their errors and output.
    pub report: Value,
}

/// `startpos.campaigns`: returns the campaigns a startpos can be built for.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct GetStartposCampaigns {

    /// Key of the pack with the startpos tables.
    pub pack: String,
}

/// Campaigns a startpos can be built for.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct StartposCampaigns {

    /// Campaign keys, sorted.
    pub campaigns: Vec<String>,

    /// Campaign the last startpos of the pack was built for, if any.
    pub last_used: Option<String>,
}

/// `startpos.start`: prepares the Assembly Kit to build a startpos with the tables of an open pack, and launches the game to build it.
///
/// When the game is closed, call [`FinishStartpos`] to import the startpos into the pack.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct StartStartpos {

    /// Key of the pack with the startpos tables.
    pub pack: String,

    /// Campaign to build the startpos for.
    pub campaign: String,

    /// If the HLP and SPD data are built too.
    #[serde(default)]
    pub process_hlp_spd_data: bool,
}

/// `startpos.finish`: imports the startpos built by the game into the pack, or cancels the build, and cleans up the build files.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct FinishStartpos {

    /// If the build is cancelled instead of imported.
    #[serde(default)]
    pub cancel: bool,
}

/// Returns the values of optimizer options, by name, as `tools.optimize` takes them.
pub fn optimizer_option_values(options: &OptimizerOptions) -> BTreeMap<String, bool> {
    match serde_json::to_value(options) {
        Ok(Value::Object(values)) => values.into_iter().filter_map(|(name, value)| value.as_bool().map(|value| (name, value))).collect(),
        _ => BTreeMap::new(),
    }
}

impl Request for GetOptimizerOptions {
    const METHOD: &'static str = "tools.optimizer_options";
    type Response = OptimizerOptionValues;
}

impl Request for OptimizePack {
    const METHOD: &'static str = "tools.optimize";
    type Response = FilesChanged;
    const IS_JOB: bool = true;
}

impl Request for PatchSiegeAi {
    const METHOD: &'static str = "tools.patch_siege_ai";
    type Response = SiegeAiPatched;
}

impl Request for PackMap {
    const METHOD: &'static str = "tools.pack_map";
    type Response = FilesChanged;
}

impl Request for GenerateMissingLocs {
    const METHOD: &'static str = "tools.generate_missing_locs";
    type Response = FilesEdited;
}

impl Request for UpdateAnimIds {
    const METHOD: &'static str = "tools.update_anim_ids";
    type Response = FilesEdited;
}

impl Request for AnimsBySkeleton {
    const METHOD: &'static str = "tools.anims_by_skeleton";
    type Response = FilePaths;
}

impl Request for ExportGltf {
    const METHOD: &'static str = "tools.export_gltf";
    type Response = Done;
}

impl Request for SetVideoFormat {
    const METHOD: &'static str = "tools.set_video_format";
    type Response = Done;
}

impl Request for LiveExport {
    const METHOD: &'static str = "tools.live_export";
    type Response = Done;
}

impl Request for InitMyMod {
    const METHOD: &'static str = "tools.init_mymod";
    type Response = MyModCreated;
}

impl Request for RunLuaTests {
    const METHOD: &'static str = "lua.run_tests";
    type Response = LuaTestResults;
    const IS_JOB: bool = true;
}

impl Request for GetStartposCampaigns {
    const METHOD: &'static str = "startpos.campaigns";
    type Response = StartposCampaigns;
}

impl Request for StartStartpos {
    const METHOD: &'static str = "startpos.start";
    type Response = Done;
}

impl Request for FinishStartpos {
    const METHOD: &'static str = "startpos.finish";
    type Response = FilesEdited;
}

/// `tools.plugin_scripts`: returns the plugin scripts in the scripts folder of the config.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListPluginScripts {}

/// Plugin scripts available.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginScripts {

    /// Paths of the scripts.
    pub scripts: Vec<PathBuf>,
}

/// `tools.run_plugin_script`: runs a plugin script over files of an open pack, reading back the files it changes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunPluginScript {

    /// Key of the pack.
    pub pack: String,

    /// Path of the script.
    pub script: PathBuf,

    /// Paths of the files and folders passed to the script.
    pub paths: Vec<String>,
}

/// Result of running a plugin script.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginScriptRun {

    /// Paths of the files read back into the pack.
    pub edited: Vec<String>,

    /// Error message of the script, if it failed.
    pub error: Option<String>,
}

/// `lua.hovers`: returns the docs of the game's Lua API for the symbols of a Lua script, to show on hover.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetLuaHovers {

    /// Source of the script.
    pub source: String,
}

/// Docs of the symbols of a Lua script. Empty if the game's Lua API is not available.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LuaHovers {

    /// The hovers.
    pub hovers: Vec<LuaHover>,
}

/// Docs of a symbol of a Lua script. Lines and columns are 0-based.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LuaHover {

    /// Line where the symbol starts.
    pub start_line: u64,

    /// Column where the symbol starts.
    pub start_column: u64,

    /// Line where the symbol ends.
    pub end_line: u64,

    /// Column where the symbol ends.
    pub end_column: u64,

    /// The docs, as rich text.
    pub docs: String,
}

/// `ceo.traits`: returns the trait CEOs of the Assembly Kit tables.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListTraitCeos {}

/// Trait CEOs of the Assembly Kit tables.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraitCeos {

    /// UUID and key of each trait CEO.
    pub traits: Vec<(String, String)>,
}

/// `ceo.add_entries`: adds CEO entries (armour, career, traits and their locs) to an open pack.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AddCeoEntries {

    /// Key of the pack.
    pub pack: String,

    /// The entries.
    pub entries: Vec<CeoEntryData>,
}

/// A CEO entry to add.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CeoEntryData {

    /// Name of the entry, used to build the keys of its rows.
    pub name: String,

    /// Kind of entry: `unique` for unique ones.
    pub option: String,

    /// Element of the entry.
    pub element: String,

    /// Gender of the entry.
    pub gender: String,

    /// UUID and key of each trait of the entry.
    pub traits: Vec<(String, String)>,

    /// If the entry also starts with a sword and shield.
    pub expanded: bool,
}

/// `ceo.build`: builds `ceo_data.ccd` in the Assembly Kit from the CEO tables of an open pack, running BOB.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildCeo {

    /// Key of the pack.
    pub pack: String,

    /// Root folder of the Assembly Kit.
    pub assembly_kit: PathBuf,

    /// Path of BOB's executable.
    pub bob: PathBuf,
}

/// `ceo.import`: imports the `ceo_data.ccd` built by `ceo.build` into an open pack.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportCeo {

    /// Key of the pack.
    pub pack: String,

    /// Root folder of the Assembly Kit.
    pub assembly_kit: PathBuf,
}

impl Request for ListPluginScripts {
    const METHOD: &'static str = "tools.plugin_scripts";
    type Response = PluginScripts;
}

impl Request for RunPluginScript {
    const METHOD: &'static str = "tools.run_plugin_script";
    type Response = PluginScriptRun;
}

impl Request for GetLuaHovers {
    const METHOD: &'static str = "lua.hovers";
    type Response = LuaHovers;
}

impl Request for ListTraitCeos {
    const METHOD: &'static str = "ceo.traits";
    type Response = TraitCeos;
}

impl Request for AddCeoEntries {
    const METHOD: &'static str = "ceo.add_entries";
    type Response = FilesEdited;
}

impl Request for BuildCeo {
    const METHOD: &'static str = "ceo.build";
    type Response = Done;
}

impl Request for ImportCeo {
    const METHOD: &'static str = "ceo.import";
    type Response = FilesEdited;
}
