//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use super::*;

/// Docs describing the few engine functions the fake library uses.
const SCRIPTING_DOC: &str = "<h2><a name=\"Event Functions\">Event Functions</a></h2>
<dl>
<dd><h4><a name=\"FactionTurnStart\">FactionTurnStart</a></h4>
<dl><dd>Function Name: faction</dd>
<dd>Interface: <a href=\"#FACTION_SCRIPT_INTERFACE\">FACTION_SCRIPT_INTERFACE</a></dd>
</dl>
<br></dd>
</dl>
<h2><a name=\"Interface Functions\">Interface Functions</a></h2>
<dl>
<h4><a name=\"FACTION_SCRIPT_INTERFACE\">FACTION_SCRIPT_INTERFACE</a></h4>
<br><dd>Function: <a name=\"FACTION_SCRIPT_INTERFACEname\">name</a></dd>
<dd>Return: String</dd>
<br>
<dd>Function: <a name=\"FACTION_SCRIPT_INTERFACEis_human\">is_human</a></dd>
<dd>Return: bool</dd>
<br>
</dl>";

const EPISODIC_PAGE: &str = "<dl class=\"function\"><dt><a name=\"function:episodic_scripting:treasury_mod\"><h3 class=\"function_name\"><code>cm:treasury_mod(<code>string</code> </strong></code><i class=\"parameter\">faction key</i><code><strong>, <code>number</code> </strong></code><i class=\"parameter\">amount</i><code><strong>)</code></h3></a></dt><dd>Changes the treasury.<h4>Returns:</h4><ol><li><code>nil</code></li></ol></dd></dl>";

/// A tiny stand-in for the game's script libraries, following the same boot flow: listeners stored in the global
/// `events` table, mods loaded on `NewSession` through `common.filesystem_lookup`, and their entry functions
/// called on the first tick.
const CAMPAIGN_SCRIPTED: &str = r#"
events = {}

core = {}
function core:add_listener(name, event, condition, callback)
    events[event] = events[event] or {}
    table.insert(events[event], function(context)
        if condition == true or condition(context) then callback(context) end
    end)
end

local loaded_mods = {}
function load_script_libraries()
    cm = setmetatable({}, { __index = function(_, key)
        return function(_, ...) local game = GAME() return game[key](game, ...) end
    end })
    cm.get_campaign_ui_manager = function() return {} end

    core:add_listener("load mods", "NewSession", true, function()
        for path in string.gmatch(common.filesystem_lookup("/script/campaign/mod/", "*.lua"), "[^,]+") do
            local name = string.match(path, "([^/]+)%.lua$")
            package.path = "/script/campaign/mod/?.lua;" .. package.path
            local chunk = assert(loadfile(name))
            chunk()
            table.insert(loaded_mods, name)
        end
    end)

    core:add_listener("run mods", "FirstTickAfterWorldCreated", true, function()
        for _, name in ipairs(loaded_mods) do
            if _G[name] then _G[name]() end
        end
    end)
end
"#;

const MOD_SCRIPT: &str = r#"
core:add_listener("kislev gold", "FactionTurnStart",
    function(context) return context:faction():name() == "wh3_main_ksl_kislev" and context:faction():is_human() end,
    function(context) cm:treasury_mod(context:faction():name(), 500) end)

function my_mod()
    print("my_mod first tick")
end
"#;

fn api() -> LuaApi {
    let mut api = LuaApi::default();
    api.add_scripting_doc(SCRIPTING_DOC);
    api.add_page(EPISODIC_PAGE, super::super::LuaEnvironment::Campaign);
    api
}

fn scripts(mod_script: &str) -> LuaScripts {
    let mut scripts = LuaScripts::default();
    scripts.add("script/campaign_scripted.lua", CAMPAIGN_SCRIPTED.to_owned(), false);
    scripts.add("script/campaign/mod/my_mod.lua", mod_script.to_owned(), true);
    scripts
}

fn run(test_source: &str) -> LuaTestReport {
    run_tests(&api(), &scripts(MOD_SCRIPT), test_source, &LuaTestOptions::default()).expect("the test file should run")
}

fn test_result<'a>(report: &'a LuaTestReport, name: &str) -> &'a LuaTestResult {
    report.tests().iter().find(|test| test.name() == name).expect("the test should be in the report")
}

#[test]
fn test_run_tests_passing_and_failing() {
    let report = run(r#"
local kislev = rpfm.faction { key = "wh3_main_ksl_kislev", is_human = true }
local empire = rpfm.faction { key = "wh_main_emp_empire", is_human = false }

rpfm.test("gives gold", function()
    rpfm.fire("FactionTurnStart", { faction = kislev })
    rpfm.assert_called("treasury_mod", "wh3_main_ksl_kislev", 500)
end)

rpfm.test("ignores others", function()
    rpfm.fire("FactionTurnStart", { faction = empire })
    rpfm.assert_not_called("treasury_mod")
end)

rpfm.test("wrong expectation", function()
    rpfm.fire("FactionTurnStart", { faction = empire })
    rpfm.assert_called("treasury_mod", "wh_main_emp_empire", 500)
end)
"#);

    assert_eq!(report.tests().len(), 3);
    assert!(test_result(&report, "gives gold").passed());
    assert!(test_result(&report, "ignores others").passed());

    let failed = test_result(&report, "wrong expectation");
    assert!(!failed.passed());
    assert!(failed.errors()[0].contains("expected a call to treasury_mod(\"wh_main_emp_empire\", 500), got no calls to it"));
    assert!(report.boot_errors().is_empty());
}

#[test]
fn test_run_tests_loads_mods() {
    let report = run("rpfm.test(\"boot\", function() end)");
    assert!(test_result(&report, "boot").log().iter().any(|line| line == "my_mod first tick"), "mod entry functions must run on the first tick");
}

#[test]
fn test_run_tests_isolates_tests() {
    let report = run(r#"
rpfm.test("sets a global", function() leaked = true end)
rpfm.test("doesn't see it", function() rpfm.assert_equal(leaked, nil) end)
"#);

    assert!(report.tests().iter().all(|test| *test.passed()));
}

#[test]
fn test_run_tests_error_attribution() {
    let broken_mod = "core:add_listener(\"broken\", \"FactionTurnStart\", true, function(context) error(\"mod broke\") end)";
    let report = run_tests(&api(), &scripts(broken_mod), "rpfm.test(\"fires\", function() rpfm.fire(\"FactionTurnStart\") end)", &LuaTestOptions::default()).expect("the test file should run");
    let result = test_result(&report, "fires");
    assert!(!result.passed());
    assert!(result.errors().iter().any(|error| error.contains("mod broke")), "errors in listeners must fail the test");

    let mut scripts = scripts(MOD_SCRIPT);
    scripts.add("script/campaign/mod/vanilla_mod.lua", "error(\"vanilla broke\")".to_owned(), false);
    let report = run_tests(&api(), &scripts, "rpfm.test(\"boot\", function() end)", &LuaTestOptions::default()).expect("the test file should run");
    assert!(test_result(&report, "boot").passed(), "errors of scripts outside the tested packs must not fail tests");
    assert!(report.boot_errors().iter().any(|error| error.contains("vanilla broke")));

    scripts.add("script/campaign/mod/broken_boot.lua", "error(\"pack broke\")".to_owned(), true);
    let report = run_tests(&api(), &scripts, "rpfm.test(\"boot\", function() end)", &LuaTestOptions::default()).expect("the test file should run");
    assert!(!test_result(&report, "boot").passed(), "errors of the tested packs while booting must fail tests");
}

#[test]
fn test_run_tests_engine_objects() {
    let mut options = LuaTestOptions::default();
    options.set_faction_keys(vec!["wh_main_emp_empire".to_owned()]);

    let report = run_tests(&api(), &scripts(MOD_SCRIPT), r#"
rpfm.test("objects", function()
    local faction = rpfm.faction { key = "wh3_main_ksl_kislev" }
    rpfm.assert_equal(string.sub(tostring(faction), 1, 24), "FACTION_SCRIPT_INTERFACE")
    rpfm.assert_equal(faction:is_human(), false, "undefined methods return defaults of their documented type")
    rpfm.assert_equal(faction.not_a_method, nil, "documented types only have documented methods")

    local world = GAME():model():world()
    rpfm.assert_equal(world:faction_by_key("wh_main_emp_empire"):is_null_interface(), false, "DB factions exist")
    rpfm.assert_equal(world:faction_by_key("missing"):is_null_interface(), true)

    local list = world:faction_list()
    rpfm.assert_equal(list:num_items(), 2)
    rpfm.assert_equal(list:item_at(0), faction)

    effect.undocumented_call()
end)
"#, &options).expect("the test file should run");

    let result = test_result(&report, "objects");
    assert!(result.passed(), "{:?}", result.errors());
    assert_eq!(*result.unmocked_calls(), vec!["effect:undocumented_call".to_owned()]);
}

#[test]
fn test_run_tests_sandbox_and_strings() {
    let report = run(r#"
rpfm.test("sandbox", function()
    rpfm.assert_equal(os.execute, nil)
    rpfm.assert_equal(io.popen, nil)
    rpfm.assert_equal(package.loadlib, nil)
    rpfm.assert_equal(io.open("missing.txt", "r"), nil)

    local file = io.open("log.txt", "w")
    file:write("a", 1)
    file:close()
    rpfm.assert_equal(io.open("log.txt", "r"):read("*a"), "a1")
end)

rpfm.test("strings", function()
    rpfm.assert_equal(string.len("añb"), 3)
    rpfm.assert_equal(string.len_lua("añb"), 4)
    rpfm.assert_equal(string.sub("añb", 2, 3), "ñb")
    rpfm.assert_equal(string.find("añb", "b"), 3)
    rpfm.assert_equal(string.find("a.b", "."), 2, "find doesn't use patterns")
end)
"#);

    for test in report.tests() {
        assert!(test.passed(), "{}: {:?}", test.name(), test.errors());
    }
}

#[test]
fn test_run_tests_invalid_test_file() {
    let result = run_tests(&api(), &scripts(MOD_SCRIPT), "rpfm.test(", &LuaTestOptions::default());
    assert!(matches!(result, Err(RLibError::LuaTestFileError(_))));
}

#[test]
fn test_lua_scripts_list() {
    let mut scripts = LuaScripts::default();
    scripts.add("script\\campaign\\mod\\B.lua", String::new(), true);
    scripts.add("script/campaign/mod/a.lua", "\u{feff}return 1".to_owned(), true);
    scripts.add("script/campaign/mod/sub/c.lua", String::new(), true);
    scripts.add("script/campaign/mod/readme.txt", String::new(), true);

    assert_eq!(scripts.list("/script/campaign/mod/", "*.lua"), "script/campaign/mod/B.lua,script/campaign/mod/a.lua");
    assert_eq!(scripts.get("SCRIPT/campaign/mod/a.lua").map(|(_, source)| source.as_str()), Some("return 1"), "lookups ignore case, and the BOM is removed");
}
