//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use std::fs;

use super::*;
use super::check::*;

/// Builds a per-page function block with the same markup as the Assembly Kit docs.
fn function_block(signature: &str, parameter_rows: &[(&str, &str, &str)], returns: &[&str]) -> String {
    let rows = parameter_rows.iter()
        .map(|(index, lua_type, description)| format!("<tr>\n<td>\n<p><strong>{index}</strong></p>\n</td>\n<td>\n<p><code><code><a href=\"lua.html#class:{lua_type}\">{lua_type}</a></code></code></p>\n</td>\n<td>\n<p>{description}</p>\n</td>\n</tr>"))
        .collect::<String>();

    let returns = returns.iter()
        .map(|lua_type| format!("<li><code>{lua_type}</code> description</li>"))
        .collect::<String>();

    format!("<dl class=\"function\">\n<dt>\n<a name=\"function:page:name\"><h3 class=\"function_name\">{signature}</h3></a>\n</dt>\n<dd>\nDescription.\n<h4>Parameters:</h4>\n<table class=\"parameter_list\">{rows}</table>\n<h4>Returns:</h4>\n<ol>\n{returns}\n</ol>\n</dd>\n</dl>\n")
}

fn return_types(function: &LuaFunction) -> Vec<LuaType> {
    function.returns().iter().map(|returned| returned.lua_type().clone()).collect()
}

fn page() -> String {
    [
        function_block(
            "<strong><code>cm:add_growth_points_to_region(<code><a href=\"lua.html#class:string\">string</a></code> </strong></code><i class=\"parameter\">region key</i><code><strong>, <br />&emsp;&emsp;[<code><a href=\"lua.html#class:number\">number</a></code> </strong></code><i class=\"parameter\">growth points</i><code><strong>])</code></strong>",
            &[("1", "string", "Region key, from the <code>campaign_map_regions</code> table."), ("2", "number", "Growth points to add.")],
            &["nil"],
        ),
        function_block(
            "<strong><code>common.ancillary(<code>string</code> </strong></code><i class=\"parameter\">ancillary key</i><code><strong>, <code>number</code> </strong></code><i class=\"parameter\">chance</i><code><strong>)</code></strong>",
            &[("1", "string", "Ancillary key."), ("2", "number", "<i>optional, default value=100</i></p>\n<p>Chance.")],
            &["boolean"],
        ),
        function_block(
            "<strong><code>script_error(<code>string</code> </strong></code><i class=\"parameter\">message</i><code><strong>, ... </strong></code><i class=\"parameter\">values</i><code><strong>)</code></strong>",
            &[],
            &["number", "faction"],
        ),
        "<dl class=\"function\">\n<dt>Not a function</dt>\n</dl>\n".to_owned(),
    ].concat()
}

const SCRIPTING_DOC: &str = "<h2>Events</h2>
<h2><a name=\"Event Functions\">Event Functions</a></h2>
<dl>
<dd><h4><a name=\"FactionTurnStart\">FactionTurnStart</a></h4>
<dl><dd>Function Name: faction</dd>
<dd>Interface: <a href=\"#FACTION_SCRIPT_INTERFACE\">FACTION_SCRIPT_INTERFACE</a></dd>
<dd>Description: Access the faction in the event</dd>
</dl>
<br></dd>
<dd><h4><a name=\"AdviceFinishedTrigger\">AdviceFinishedTrigger</a></h4>
</dd>
</dl>
<h2><a name=\"Interface Functions\">Interface Functions</a></h2>
<dl>
<h4><a name=\"FACTION_SCRIPT_INTERFACE\">FACTION_SCRIPT_INTERFACE</a></h4>
<dd>Description: Faction interface</dd>
<br>
<dd><a href=\"#FACTION_SCRIPT_INTERFACEname\">name</a></dd>
<br><dd>Function: <a name=\"FACTION_SCRIPT_INTERFACEname\">name</a></dd>
<dd>Description: The faction key</dd>
<dd>Parameters: name()</dd>
<dd>Return: String</dd>
<br>
<dd>Function: <a name=\"FACTION_SCRIPT_INTERFACEregion_list\">region_list</a></dd>
<dd>Description: A list of regions owned by the faction</dd>
<dd>Parameters: region_list()</dd>
<dd>Return: <a href=\"#REGION_LIST_SCRIPT_INTERFACE\">REGION_LIST_SCRIPT_INTERFACE</a></dd>
<br>
<dd>Function: <a name=\"FACTION_SCRIPT_INTERFACEwas_confederated\">was_confederated</a></dd>
<dd>Description: Returns if the faction was confederated</dd>
<dd>Return: <a href=\"#FACTION_SCRIPT_INTERFACE\">FACTION_SCRIPT_INTERFACE</a></dd>
<br>
<dd>Function: <a name=\"FACTION_SCRIPT_INTERFACEreset\">reset</a></dd>
<dd>Description: Does something</dd>
<dd>Parameters: reset()</dd>
<dd>Return: void</dd>
<br>
</dl>";

#[test]
fn test_lua_type_from_doc_name() {
    let cases = [
        ("boolean", LuaType::Boolean),
        ("bool", LuaType::Boolean),
        ("logical", LuaType::Boolean),
        ("card32", LuaType::Number),
        ("positive int", LuaType::Number),
        ("100 >= float >= 0", LuaType::Number),
        ("String", LuaType::String),
        ("String table", LuaType::Table),
        ("Table(String)", LuaType::Table),
        ("function", LuaType::Function),
        ("void", LuaType::Nil),
        ("NONE", LuaType::Nil),
        ("", LuaType::Nil),
        ("userdata", LuaType::Any),
        ("(X, Y)", LuaType::Any),
        ("FACTION_SCRIPT_INTERFACE", LuaType::Interface("FACTION_SCRIPT_INTERFACE".to_owned())),
        ("faction", LuaType::Object("faction".to_owned())),
        ("uicomponent/function", LuaType::Object("uicomponent".to_owned())),
    ];

    for (doc_name, expected) in cases {
        assert_eq!(LuaType::from_doc_name(doc_name), expected, "doc name: {doc_name}");
    }
}

#[test]
fn test_html_to_text() {
    assert_eq!(html_to_text("<code>a &amp; b</code>&emsp;&lt;c&gt;&#65;&#x42;&unknown;"), "a & b <c>AB&unknown;");
    assert_eq!(html_to_text("  multiple<br />\n\t  spaces "), "multiple spaces");
}

#[test]
fn test_add_page_method() {
    let mut api = LuaApi::default();
    api.add_page(&page(), LuaEnvironment::Campaign);

    let function = api.function("cm", "add_growth_points_to_region").expect("method should be parsed");
    assert_eq!(*function.call_style(), LuaCallStyle::Method);
    assert!(function.returns().is_empty());
    assert_eq!(function.environments().iter().collect::<Vec<_>>(), vec![&LuaEnvironment::Campaign]);

    let parameters = function.parameters().as_ref().expect("parameters should be parsed");
    assert_eq!(parameters.len(), 2);
    assert_eq!(parameters[0].name(), "region key");
    assert_eq!(*parameters[0].lua_type(), LuaType::String);
    assert!(!parameters[0].optional());
    assert_eq!(parameters[0].db_table().as_deref(), Some("campaign_map_regions_tables"));
    assert_eq!(parameters[1].name(), "growth points");
    assert_eq!(*parameters[1].lua_type(), LuaType::Number);
    assert!(parameters[1].optional());
    assert!(parameters[1].db_table().is_none());
}

#[test]
fn test_add_page_descriptions() {
    let mut api = LuaApi::default();
    api.add_page(&page(), LuaEnvironment::Campaign);

    let function = api.function("cm", "add_growth_points_to_region").expect("method should be parsed");
    assert_eq!(function.signature(), "cm:add_growth_points_to_region(string region key, [number growth points])");
    assert_eq!(function.description(), "Description.");

    let parameters = function.parameters().as_ref().expect("parameters should be parsed");
    assert_eq!(parameters[0].description(), "Region key, from the campaign_map_regions table.");

    let global = api.function("", "script_error").expect("global function should be parsed");
    assert_eq!(global.returns()[1].description(), "description");
}

#[test]
fn test_add_page_field_and_global() {
    let mut api = LuaApi::default();
    api.add_page(&page(), LuaEnvironment::Battle);

    let field = api.function("common", "ancillary").expect("field function should be parsed");
    assert_eq!(*field.call_style(), LuaCallStyle::Field);
    assert_eq!(return_types(field), vec![LuaType::Boolean]);

    // The optional marker can come from the parameter description instead of the signature.
    let parameters = field.parameters().as_ref().expect("parameters should be parsed");
    assert!(!parameters[0].optional());
    assert!(parameters[1].optional());
    assert_eq!(parameters[1].description(), "(optional, default: 100) Chance.");

    let global = api.function("", "script_error").expect("global function should be parsed");
    assert_eq!(*global.call_style(), LuaCallStyle::Global);
    assert_eq!(return_types(global), vec![LuaType::Number, LuaType::Object("faction".to_owned())]);

    let parameters = global.parameters().as_ref().expect("parameters should be parsed");
    assert_eq!(parameters.len(), 2);
    assert!(!parameters[0].variadic());
    assert!(parameters[1].variadic());
    assert_eq!(*parameters[1].lua_type(), LuaType::Any);

    let functions = api.owners().values().map(|functions| functions.len()).sum::<usize>();
    assert_eq!(functions, 3, "blocks without a function anchor must be skipped");
}

#[test]
fn test_add_page_merges_environments() {
    let mut api = LuaApi::default();
    api.add_page(&page(), LuaEnvironment::Campaign);
    api.add_page(&page(), LuaEnvironment::Battle);

    let function = api.function("common", "ancillary").expect("function should be parsed");
    assert_eq!(function.environments().iter().copied().collect::<Vec<_>>(), vec![LuaEnvironment::Campaign, LuaEnvironment::Battle]);
}

#[test]
fn test_add_scripting_doc() {
    let mut api = LuaApi::default();
    api.add_scripting_doc(SCRIPTING_DOC);

    let accessors = api.events().get("FactionTurnStart").expect("event should be parsed");
    let faction = accessors.get("faction").expect("accessor should be parsed");
    assert_eq!(*faction.lua_type(), LuaType::Interface("FACTION_SCRIPT_INTERFACE".to_owned()));
    assert_eq!(faction.description(), "Access the faction in the event");
    assert!(api.events().get("AdviceFinishedTrigger").expect("events without accessors should be parsed").is_empty());

    let name = api.function("FACTION_SCRIPT_INTERFACE", "name").expect("interface method should be parsed");
    assert_eq!(*name.call_style(), LuaCallStyle::Method);
    assert_eq!(return_types(name), vec![LuaType::String]);
    assert_eq!(name.signature(), "FACTION_SCRIPT_INTERFACE:name()");
    assert_eq!(name.description(), "The faction key");
    assert!(name.parameters().is_none());
    assert!(name.environments().is_empty());

    let region_list = api.function("FACTION_SCRIPT_INTERFACE", "region_list").expect("interface method should be parsed");
    assert_eq!(return_types(region_list), vec![LuaType::Interface("REGION_LIST_SCRIPT_INTERFACE".to_owned())]);

    let was_confederated = api.function("FACTION_SCRIPT_INTERFACE", "was_confederated").expect("interface method should be parsed");
    assert_eq!(return_types(was_confederated), vec![LuaType::Boolean], "known doc errors must be corrected");

    let reset = api.function("FACTION_SCRIPT_INTERFACE", "reset").expect("interface method should be parsed");
    assert!(reset.returns().is_empty());
}

#[test]
fn test_from_assembly_kit() {
    let docs_path = std::env::temp_dir().join(format!("rpfm_lua_api_test_{}", std::process::id()));
    fs::create_dir_all(docs_path.join("campaign")).expect("temp folder should be writable");
    fs::write(docs_path.join("campaign").join("episodic_scripting.html"), page()).expect("temp file should be writable");
    fs::write(docs_path.join("campaign").join("notes.txt"), "not a page").expect("temp file should be writable");
    fs::write(docs_path.join(SCRIPTING_DOC_FILE), SCRIPTING_DOC).expect("temp file should be writable");

    let api = LuaApi::from_assembly_kit(&docs_path);
    fs::remove_dir_all(&docs_path).expect("temp folder should be removable");

    let api = api.expect("docs should be loaded");
    assert!(api.function("cm", "add_growth_points_to_region").is_some());
    assert!(api.function("FACTION_SCRIPT_INTERFACE", "name").is_some());
    assert!(api.events().contains_key("FactionTurnStart"));
}

#[test]
fn test_from_assembly_kit_missing_folder() {
    let result = LuaApi::from_assembly_kit(Path::new("/this/path/does/not/exist"));
    assert!(matches!(result, Err(RLibError::AssemblyKitNotFound)));
}

//---------------------------------------------------------------------------//
//                                Checker tests
//---------------------------------------------------------------------------//

fn api() -> LuaApi {
    let mut api = LuaApi::default();
    api.add_page(&page(), LuaEnvironment::Campaign);
    api.add_scripting_doc(SCRIPTING_DOC);
    api
}

/// Checks a script with the fixture API, using the script's own definitions.
fn check(source: &str) -> LuaCheckResult {
    let mut definitions = LuaDefinitions::default();
    definitions.add_script(source);
    check_script(source, Some(&api()), &definitions)
}

fn kinds(result: &LuaCheckResult) -> Vec<LuaIssueKind> {
    result.issues().iter().map(|issue| issue.kind().clone()).collect()
}

#[test]
fn test_check_syntax_error() {
    let result = check_script("local x = \nlocal y = 1", None, &LuaDefinitions::default());
    assert_eq!(result.issues().len(), 1);
    assert!(matches!(result.issues()[0].kind(), LuaIssueKind::SyntaxError(_)));
    assert!(result.key_references().is_empty());
}

#[test]
fn test_check_without_api_only_checks_syntax() {
    let result = check_script("cm:not_a_method()", None, &LuaDefinitions::default());
    assert!(result.issues().is_empty());
}

#[test]
fn test_check_unknown_method_and_position() {
    let result = check("\n   cm:add_growth_points_to_regoin(\"a\")");
    assert_eq!(kinds(&result), vec![LuaIssueKind::UnknownMethod("cm".to_owned(), "add_growth_points_to_regoin".to_owned())]);

    let (start, end) = *result.issues()[0].range();
    assert_eq!(start, (1, 6));
    assert_eq!(end.0, 1);
    assert!(end.1 > start.1);
}

#[test]
fn test_check_argument_count() {
    let method = |args: &str| kinds(&check(&format!("cm:add_growth_points_to_region({args})")));
    assert_eq!(method(""), vec![LuaIssueKind::WrongArgumentCount("add_growth_points_to_region".to_owned(), 1, Some(2), 0)]);
    assert_eq!(method("\"a\", 1, 2"), vec![LuaIssueKind::WrongArgumentCount("add_growth_points_to_region".to_owned(), 1, Some(2), 3)]);
    assert!(method("\"a\"").is_empty());
    assert!(method("\"a\", 1").is_empty());
    assert!(method("get_values()").is_empty(), "a call as last argument can expand to several values");

    assert_eq!(kinds(&check("common.ancillary()")), vec![LuaIssueKind::WrongArgumentCount("ancillary".to_owned(), 1, Some(2), 0)]);
    assert!(kinds(&check("common.ancillary \"key\"")).is_empty());

    assert_eq!(kinds(&check("script_error()")), vec![LuaIssueKind::WrongArgumentCount("script_error".to_owned(), 1, None, 0)]);
    assert!(kinds(&check("script_error(\"a\", 1, 2, 3)")).is_empty());
}

#[test]
fn test_check_key_references() {
    let result = check("cm:add_growth_points_to_region(\"wh3_main_region\", 5)\ncm:add_growth_points_to_region(region_variable)\ncm:add_growth_points_to_region(\"\")");
    assert!(result.issues().is_empty());
    assert_eq!(result.key_references().len(), 1);

    let reference = &result.key_references()[0];
    assert_eq!(reference.table(), "campaign_map_regions_tables");
    assert_eq!(reference.key(), "wh3_main_region");
    assert_eq!(*reference.range(), ((0, 32), (0, 47)), "the range must not include the quotes");
}

#[test]
fn test_check_listeners() {
    let source = "
core:add_listener(\"l\", \"FactionTurnStart\", true, function(context)
    local faction = context:faction()
    faction:name()
    faction:nmae()
    context:facton()
end, true)
core:add_listener(\"l\", \"NotAnEvent\", true, function() end, true)
core:add_listener(\"l\", \"AdviceFinishedTrigger\", true, function(context) context:anything() end, true)
core:add_listener(\"l\", \"ScriptEventMine\", true, function() end, true)
core:add_listener(\"l\", \"MyTriggered\", true, function() end, true)
core:trigger_event(\"MyTriggered\")
";

    assert_eq!(kinds(&check(source)), vec![
        LuaIssueKind::UnknownMethod("FACTION_SCRIPT_INTERFACE".to_owned(), "nmae".to_owned()),
        LuaIssueKind::UnknownMethod("FactionTurnStart context".to_owned(), "facton".to_owned()),
        LuaIssueKind::UnknownEvent("NotAnEvent".to_owned()),
    ]);
}

#[test]
fn test_check_scopes() {
    let source = "
local cm = {}
cm:whatever()
core:add_listener(\"l\", \"FactionTurnStart\", true, function(context)
    local faction = context:faction()
    faction = get_something_else()
    faction:nmae()
end, true)
";

    assert!(kinds(&check(source)).is_empty(), "shadowed and reassigned variables must not be checked");
}

#[test]
fn test_check_definitions() {
    let source = "function cm:my_helper() end\ncm.my_value = function() end\ncm:my_helper()\ncm:my_value()";
    assert!(kinds(&check(source)).is_empty());

    let result = check_script("cm:my_helper()", Some(&api()), &LuaDefinitions::default());
    assert_eq!(kinds(&result), vec![LuaIssueKind::UnknownMethod("cm".to_owned(), "my_helper".to_owned())]);

    let mut api = api();
    api.script_definitions.add_script("function campaign_manager:vanilla_helper() end");
    let result = check_script("cm:vanilla_helper()", Some(&api), &LuaDefinitions::default());
    assert!(kinds(&result).is_empty(), "members defined by vanilla scripts must not be reported");
}

#[test]
fn test_definitions_collector() {
    let mut definitions = LuaDefinitions::default();
    definitions.add_script("function a.b.c() end\nfunction d:e() end\nf.g = 1\ncore:trigger_custom_event(\"Custom\", {})\ninvalid syntax here(");
    assert!(definitions.members().is_empty(), "scripts with syntax errors are ignored");

    definitions.add_script("function a.b.c() end\nfunction d:e() end\nf.g = 1\ncore:trigger_custom_event(\"Custom\", {})");
    for member in [("b", "c"), ("d", "e"), ("f", "g")] {
        assert!(definitions.members().contains(&(member.0.to_owned(), member.1.to_owned())), "missing {member:?}");
    }
    assert!(definitions.events().contains("Custom"));
}

#[test]
fn test_add_events_script() {
    let mut api = api();
    api.add_events_script("module(..., package.seeall)\n\n-- Event tables\n\nAdviceCleared = {}\nFactionTurnStart = {}\nnot_an_event = 5\n");

    assert!(api.events().get("AdviceCleared").is_some_and(HashMap::is_empty));
    assert_eq!(api.events().get("FactionTurnStart").map(HashMap::len), Some(1), "documented accessors must be kept");
    assert!(!api.events().contains_key("not_an_event"));
}

#[test]
fn test_hover_html() {
    let api = api();

    let function = api.hover_html(&LuaHoverTarget::Function("cm".to_owned(), "add_growth_points_to_region".to_owned())).expect("function should have docs");
    assert!(function.starts_with("<p><code>cm:add_growth_points_to_region(string region key, [number growth points])</code></p>"));
    assert!(function.contains("<li><code>region key</code>: Region key, from the campaign_map_regions table.</li>"));
    assert!(!function.contains("Returns"), "functions returning nothing must not list returns");

    let accessor = api.hover_html(&LuaHoverTarget::Accessor("FactionTurnStart".to_owned(), "faction".to_owned())).expect("accessor should have docs");
    assert!(accessor.contains("<code>FACTION_SCRIPT_INTERFACE</code>"));

    let event = api.hover_html(&LuaHoverTarget::Event("FactionTurnStart".to_owned())).expect("event should have docs");
    assert!(event.contains("<code>context:faction()</code>"));

    assert!(api.hover_html(&LuaHoverTarget::Function("cm".to_owned(), "missing".to_owned())).is_none());
    assert!(api.hover_html(&LuaHoverTarget::Event("Missing".to_owned())).is_none());
}

#[test]
fn test_hover_html_escapes_docs() {
    let mut api = LuaApi::default();
    api.add_page(&function_block("<code>cm:compare(<code>string</code> </strong></code><i class=\"parameter\">a</i><code><strong>)</code>", &[("1", "string", "Must be &lt;b&gt; &amp; not bold.")], &["boolean"]), LuaEnvironment::Campaign);

    let html = api.hover_html(&LuaHoverTarget::Function("cm".to_owned(), "compare".to_owned())).expect("function should have docs");
    assert!(html.contains("Must be &lt;b&gt; &amp; not bold."));
}

#[test]
fn test_check_hovers() {
    let source = "core:add_listener(\"l\", \"FactionTurnStart\", true, function(context)\n    context:faction():name()\nend, true)\ncm:add_growth_points_to_region(\"a\")\nscript_error(\"a\")";
    let result = check(source);
    let hovers = result.hovers().iter().map(|hover| (hover.target().clone(), *hover.range())).collect::<Vec<_>>();

    assert!(hovers.contains(&(LuaHoverTarget::Event("FactionTurnStart".to_owned()), ((0, 24), (0, 40)))));
    assert!(hovers.contains(&(LuaHoverTarget::Accessor("FactionTurnStart".to_owned(), "faction".to_owned()), ((1, 12), (1, 19)))));
    assert!(hovers.contains(&(LuaHoverTarget::Function("FACTION_SCRIPT_INTERFACE".to_owned(), "name".to_owned()), ((1, 22), (1, 26)))));
    assert!(hovers.contains(&(LuaHoverTarget::Function("cm".to_owned(), "add_growth_points_to_region".to_owned()), ((3, 3), (3, 30)))));
    assert!(hovers.contains(&(LuaHoverTarget::Function(String::new(), "script_error".to_owned()), ((4, 0), (4, 12)))));

    let unique = hovers.iter().collect::<std::collections::HashSet<_>>();
    assert_eq!(unique.len(), hovers.len(), "chains checked twice must not duplicate hovers");
}

#[test]
fn test_vanilla_scripts() {
    use rpfm_lib::files::{FileType, RFile};
    use crate::dependencies::Dependencies;

    let dependencies = Dependencies::new_with_vanilla_files(vec![
        RFile::new_from_vec(b"return 1", FileType::Text, 0, "script/_lib/lib_core.lua"),
        RFile::new_from_vec(b"", FileType::Text, 0, "script/events.lua"),
        RFile::new_from_vec(b"", FileType::Text, 0, "script/readme.txt"),
        RFile::new_from_vec(b"", FileType::Text, 0, "scripts_other/other.lua"),
    ]);

    let mut scripts = vanilla_scripts(&dependencies);
    scripts.sort();
    assert_eq!(scripts, vec![
        ("script/_lib/lib_core.lua".to_owned(), "return 1".to_owned()),
        ("script/events.lua".to_owned(), String::new()),
    ]);
}
