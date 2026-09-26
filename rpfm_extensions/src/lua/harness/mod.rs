//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Test harness running Lua scripts outside of the game.
//!
//! Scripts run in Lua 5.1, the version the games use, together with the real script libraries of the game
//! (`script/_lib`) and, optionally, the vanilla scripts of a campaign. What the game's C++ side provides is
//! emulated by `engine.lua`: objects typed after the scripting docs that record every call made on them.
//!
//! A test file registers tests with `rpfm.test(name, function)`. Its top-level code runs before the game
//! boots, so the world it sets up (`rpfm.faction { key = "...", is_human = true }`) exists when the scripts
//! start. Each test then runs in its own Lua state, after the libraries, the mods of the tested packs and the
//! first tick have run, so tests can't affect each other.
//!
//! Scripts are sandboxed: they can't run programs, load native code or touch files outside of memory.

use getset::{Getters, Setters};
use mlua::{HookTriggers, Lua, LuaOptions, StdLib, Table, VmState};
use serde_derive::{Deserialize, Serialize};

use std::cell::Cell;
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;
use std::sync::Arc;

use rpfm_lib::error::{RLibError, Result};
use rpfm_lib::files::{Container, FileType, pack::Pack, RFileDecoded};

use crate::dependencies::Dependencies;

use super::{vanilla_scripts, LuaApi, LuaType};

#[cfg(test)] mod tests;

/// Emulation of the game's engine, and the test API.
const ENGINE_SCRIPT: &str = include_str!("engine.lua");

/// Script the game runs first when loading a campaign, which loads everything else.
const CAMPAIGN_BOOT_SCRIPT: &str = "script/campaign_scripted.lua";

/// Name of the chunk holding the test file, as shown in errors.
const TEST_FILE_CHUNK: &str = "@test";

/// Instructions a Lua state may run, boot included, before it's considered stuck.
const INSTRUCTION_LIMIT: u64 = 2_000_000_000;

/// How often, in instructions, the instruction limit is checked.
const INSTRUCTION_CHECK_INTERVAL: u32 = 100_000;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// Scripts the harness can load, by path.
///
/// Shared behind [`Arc`]s, as every test's Lua state gets its own copy.
#[derive(Clone, Debug, Default)]
pub struct LuaScripts {

    /// Path and code of each script, by lowercase path, as file lookups in the game are case-insensitive.
    scripts: Arc<HashMap<String, (String, String)>>,

    /// Lowercase paths of the scripts coming from the tested packs.
    pack_paths: Arc<Vec<String>>,
}

/// Options of a test run.
#[derive(Clone, Debug, Default, PartialEq, Getters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", set = "pub")]
pub struct LuaTestOptions {

    /// Campaign whose vanilla scripts to load, like `main_warhammer`. If `None`, only the script libraries and
    /// the mods are loaded, which is faster and less affected by engine behavior the harness doesn't emulate.
    campaign: Option<String>,

    /// Keys of the factions in the game's DB. Scripts looking them up get a faction even if tests didn't set it up.
    faction_keys: Vec<String>,

    /// Keys of the regions in the game's DB. Scripts looking them up get a region even if tests didn't set it up.
    region_keys: Vec<String>,
}

/// Results of running a test file.
#[derive(Clone, Debug, Default, PartialEq, Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct LuaTestReport {

    /// Result of each test, in the order they were registered.
    tests: Vec<LuaTestResult>,

    /// Errors raised while booting by scripts that don't come from the tested packs, like vanilla scripts
    /// reacting to engine behavior the harness doesn't emulate. They don't fail tests.
    boot_errors: Vec<String>,
}

/// Result of a test.
#[derive(Clone, Debug, Default, PartialEq, Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct LuaTestResult {

    /// Name of the test.
    name: String,

    /// If the test passed.
    passed: bool,

    /// Why the test failed: its own errors, errors in listeners while it ran, and errors of the tested
    /// packs' scripts while booting.
    errors: Vec<String>,

    /// Methods called during the test that the docs don't describe, as `type:method`. Their return values
    /// are placeholders, so results depending on them may not match the game.
    unmocked_calls: Vec<String>,

    /// Output of the scripts during the whole run, boot included. Only the latest lines are kept for long runs.
    log: Vec<String>,
}

//---------------------------------------------------------------------------//
//                             Implementations
//---------------------------------------------------------------------------//

impl LuaScripts {

    /// This function collects the vanilla scripts from the dependencies cache, and the scripts of the tested packs.
    ///
    /// # Arguments
    ///
    /// * `dependencies` - Dependencies cache with the vanilla files loaded.
    /// * `packs` - Packs to test. Their scripts replace vanilla scripts with the same path.
    ///
    /// # Returns
    ///
    /// The scripts.
    pub fn from_game_and_packs(dependencies: &Dependencies, packs: &BTreeMap<String, Pack>) -> Self {
        let mut scripts = Self::default();
        for (path, source) in vanilla_scripts(dependencies) {
            scripts.add(&path, source, false);
        }

        for file in packs.values().flat_map(|pack| pack.files_by_type(&[FileType::Text])) {
            let path = file.path_in_container_raw();
            if !path.ends_with(".lua") {
                continue;
            }

            // Scripts of open packs may be edited in memory, so their decoded text takes priority over their data.
            let source = match file.decoded() {
                Ok(RFileDecoded::Text(text)) => Some(text.contents().to_owned()),
                _ => {
                    let mut file = file.clone();
                    file.load().ok().and_then(|_| file.cached().ok().map(|data| String::from_utf8_lossy(data).to_string()))
                }
            };

            if let Some(source) = source {
                scripts.add(path, source, true);
            }
        }

        scripts
    }

    /// This function adds a script. Scripts added later replace earlier ones with the same path.
    ///
    /// # Arguments
    ///
    /// * `path` - Path of the script, like `script/campaign/mod/my_mod.lua`.
    /// * `source` - Code of the script.
    /// * `from_tested_pack` - If the script belongs to one of the packs being tested.
    pub fn add(&mut self, path: &str, source: String, from_tested_pack: bool) {
        let path = path.replace('\\', "/");
        let key = path.to_lowercase();
        let source = source.strip_prefix('\u{feff}').map(str::to_owned).unwrap_or(source);
        if from_tested_pack {
            Arc::make_mut(&mut self.pack_paths).push(key.to_owned());
        }

        Arc::make_mut(&mut self.scripts).insert(key, (path, source));
    }

    /// This function returns the path and code of a script.
    fn get(&self, path: &str) -> Option<&(String, String)> {
        self.scripts.get(&path.to_lowercase())
    }

    /// This function returns the paths of the scripts directly inside a folder, like the game's `filesystem_lookup`.
    ///
    /// # Arguments
    ///
    /// * `folder` - Folder to look in, like `script/campaign/mod/`.
    /// * `filter` - Filter for the file names. Only `*` and `*.extension` are supported.
    ///
    /// # Returns
    ///
    /// The paths, sorted and separated by commas.
    fn list(&self, folder: &str, filter: &str) -> String {
        let folder = folder.trim_matches('/').to_lowercase() + "/";
        let extension = filter.strip_prefix("*.").filter(|extension| *extension != "*").map(|extension| format!(".{}", extension.to_lowercase()));

        let mut paths = self.scripts.iter()
            .filter(|(key, _)| key.strip_prefix(&folder).is_some_and(|name| !name.contains('/')))
            .filter(|(key, _)| extension.as_ref().is_none_or(|extension| key.ends_with(extension)))
            .map(|(_, (path, _))| path.as_str())
            .collect::<Vec<_>>();

        paths.sort_unstable();
        paths.join(",")
    }

    /// This function checks if an error comes from a script of the tested packs.
    ///
    /// Lua errors start with the path of the script that raised them.
    fn is_pack_error(&self, error: &str) -> bool {
        let error = error.to_lowercase();
        self.pack_paths.iter().any(|path| error.contains(path.as_str()))
    }
}

/// This function runs the tests of a test file.
///
/// # Arguments
///
/// * `api` - API of the game, whose docs type the engine objects.
/// * `scripts` - Scripts of the game and of the tested packs.
/// * `test_source` - Code of the test file.
/// * `options` - Options of the run.
///
/// # Returns
///
/// The result of each test.
///
/// # Errors
///
/// Returns an error if the game's scripts are missing, if the test file can't be loaded, or if the Lua runtime fails.
pub fn run_tests(api: &LuaApi, scripts: &LuaScripts, test_source: &str, options: &LuaTestOptions) -> Result<LuaTestReport> {
    if scripts.get(CAMPAIGN_BOOT_SCRIPT).is_none() {
        return Err(RLibError::LuaTestGameScriptsNotFound);
    }

    let mut report = LuaTestReport::default();
    let mut index = 1;

    // Every test gets a fresh state, so the first one also tells how many tests there are.
    loop {
        let lua = new_state(api, scripts, options)?;
        load_test_file(&lua, test_source)?;

        let rpfm: Table = lua.globals().get("rpfm").map_err(lua_error)?;
        let tests: Table = rpfm.get("tests").map_err(lua_error)?;
        if index > tests.raw_len() {
            break;
        }

        let boot: mlua::Function = rpfm.get("__boot").map_err(lua_error)?;
        let boot_error = boot.call::<Option<String>>(options.campaign.as_deref()).map_err(lua_error)?;

        let (pack_errors, other_errors): (Vec<_>, Vec<_>) = string_list(&rpfm, "errors")?.into_iter().partition(|error| scripts.is_pack_error(error));
        if index == 1 {
            report.boot_errors = other_errors;
        }

        let test: Table = tests.get(index).map_err(lua_error)?;
        let name: String = test.get("name").map_err(lua_error)?;

        let run_test: mlua::Function = rpfm.get("__run_test").map_err(lua_error)?;
        let (passed, test_error): (bool, Option<String>) = match run_test.call::<(bool, Option<String>)>(index) {
            Ok(result) => result,
            Err(error) => (false, Some(error.to_string())),
        };

        // Without the libraries nothing else works, so that's what every test must report.
        let mut errors = boot_error.map(|error| vec![format!("The game's script libraries failed to load: {error}")]).unwrap_or_default();
        errors.extend(pack_errors);
        errors.extend(test_error);
        errors.extend(string_list(&rpfm, "errors")?);

        let mut unmocked_calls = rpfm.get::<Table>("unmocked").map_err(lua_error)?
            .pairs::<String, bool>()
            .filter_map(|pair| pair.ok().map(|(call, _)| call))
            .collect::<Vec<_>>();
        unmocked_calls.sort_unstable();

        report.tests.push(LuaTestResult {
            name,
            passed: passed && errors.is_empty(),
            errors,
            unmocked_calls,
            log: string_list(&rpfm, "log")?,
        });

        index += 1;
    }

    Ok(report)
}

/// This function creates a sandboxed Lua state with the engine emulation loaded.
fn new_state(api: &LuaApi, scripts: &LuaScripts, options: &LuaTestOptions) -> Result<Lua> {

    // SAFETY: the debug library can break Lua's memory safety guarantees through functions like `debug.setupvalue`
    // on C functions. The game's scripts need `debug.traceback` and `debug.getinfo`, and they only run in this state.
    let lua = unsafe { Lua::unsafe_new_with(StdLib::ALL_SAFE | StdLib::DEBUG, LuaOptions::default()) };

    let instructions = Rc::new(Cell::new(0u64));
    lua.set_hook(HookTriggers::new().every_nth_instruction(INSTRUCTION_CHECK_INTERVAL), move |_, _| {
        instructions.set(instructions.get() + u64::from(INSTRUCTION_CHECK_INTERVAL));
        if instructions.get() > INSTRUCTION_LIMIT {
            return Err(mlua::Error::runtime("the scripts ran for too long, they may be stuck in a loop"));
        }
        Ok(VmState::Continue)
    }).map_err(lua_error)?;

    let globals = lua.globals();
    let read_scripts = scripts.clone();
    let read_script = lua.create_function(move |_, path: String| Ok(read_scripts.get(&path).map(|(_, source)| source.to_owned()))).map_err(lua_error)?;
    let list_scripts_map = scripts.clone();
    let list_scripts = lua.create_function(move |_, (folder, filter): (String, String)| Ok(list_scripts_map.list(&folder, &filter))).map_err(lua_error)?;

    globals.set("rpfm_read_script", read_script).map_err(lua_error)?;
    globals.set("rpfm_list_scripts", list_scripts).map_err(lua_error)?;
    globals.set("rpfm_returns", returns_table(&lua, api)?).map_err(lua_error)?;
    globals.set("rpfm_accessors", accessors_table(&lua, api)?).map_err(lua_error)?;
    globals.set("rpfm_faction_keys", key_set(&lua, options.faction_keys())?).map_err(lua_error)?;
    globals.set("rpfm_region_keys", key_set(&lua, options.region_keys())?).map_err(lua_error)?;

    lua.load(ENGINE_SCRIPT).set_name("@engine").exec().map_err(lua_error)?;
    Ok(lua)
}

/// This function runs the top-level code of a test file, which registers its tests and sets up the world.
fn load_test_file(lua: &Lua, test_source: &str) -> Result<()> {
    lua.load(test_source).set_name(TEST_FILE_CHUNK).exec().map_err(|error| RLibError::LuaTestFileError(error.to_string()))
}

/// This function builds the table of return types of documented methods, by owner, then by method.
///
/// Only the first returned value is used. Types are passed as their names, with objects resolved to the
/// game interface of the same name when there is one.
fn returns_table(lua: &Lua, api: &LuaApi) -> Result<Table> {
    let owners = lua.create_table().map_err(lua_error)?;
    for (owner, functions) in api.owners() {
        let methods = lua.create_table().map_err(lua_error)?;
        for (name, function) in functions {
            let return_type = function.returns().first().map_or_else(|| "nil".to_owned(), |returned| type_name(api, returned.lua_type()));
            methods.set(name.as_str(), return_type).map_err(lua_error)?;
        }
        owners.set(owner.as_str(), methods).map_err(lua_error)?;
    }

    Ok(owners)
}

/// This function builds the table of return types of context accessors, by event, then by accessor.
fn accessors_table(lua: &Lua, api: &LuaApi) -> Result<Table> {
    let events = lua.create_table().map_err(lua_error)?;
    for (event, accessors) in api.events() {
        let types = lua.create_table().map_err(lua_error)?;
        for (name, accessor) in accessors {
            types.set(name.as_str(), type_name(api, accessor.lua_type())).map_err(lua_error)?;
        }
        events.set(event.as_str(), types).map_err(lua_error)?;
    }

    Ok(events)
}

/// This function builds a set of keys, as a table with each key set to `true`.
fn key_set(lua: &Lua, keys: &[String]) -> Result<Table> {
    let set = lua.create_table_with_capacity(0, keys.len()).map_err(lua_error)?;
    for key in keys {
        set.set(key.as_str(), true).map_err(lua_error)?;
    }

    Ok(set)
}

/// This function returns the name the engine emulation uses for a type.
fn type_name(api: &LuaApi, lua_type: &LuaType) -> String {
    match lua_type {
        LuaType::Object(object) => {
            let interface = format!("{}_SCRIPT_INTERFACE", object.to_uppercase());
            if api.owners().contains_key(&interface) { interface } else { "any".to_owned() }
        }
        _ => lua_type.name().to_owned(),
    }
}

/// This function reads a list of strings from a field of a table.
fn string_list(table: &Table, field: &str) -> Result<Vec<String>> {
    let list: Table = table.get(field).map_err(lua_error)?;
    list.sequence_values::<String>().collect::<mlua::Result<Vec<_>>>().map_err(lua_error)
}

/// This function turns a Lua error into an RPFM error.
fn lua_error(error: mlua::Error) -> RLibError {
    RLibError::LuaRuntimeError(error.to_string())
}
