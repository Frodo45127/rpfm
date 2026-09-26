//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Knowledge about the Lua scripting API of the games.
//!
//! The API description is not shipped with RPFM. It's built at runtime from the scripting documentation
//! bundled with the Assembly Kit (`documentation/script/script/`), which has two formats:
//!
//! - Per-page docs (`campaign/*.html`, `battle/*.html`, `frontend/*.html`): one `<dl class="function">`
//!   block per function, with a typed signature, a parameter table and a list of return values.
//! - `scripting_doc.html`: the game interfaces (`FACTION_SCRIPT_INTERFACE`, ...) with the return type of
//!   each of their methods, and the context accessors available on each event.
//!
//! The docs don't list every event, nor what the vanilla scripts define beyond the documented libraries,
//! so both are completed from the vanilla scripts in the dependencies cache. The [`check`] submodule uses
//! this description to check scripts.

use getset::Getters;
use rayon::prelude::*;
use regex::Regex;

use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::Path;
use std::sync::LazyLock;

use rpfm_lib::error::{RLibError, Result};
use rpfm_lib::files::ContainerPath;

use crate::dependencies::Dependencies;

use self::check::LuaDefinitions;

pub mod check;
#[cfg(test)] mod tests;

/// Path of the scripting documentation, relative to the root folder of the Assembly Kit.
pub const ASSEMBLY_KIT_SCRIPT_DOCS_PATH: &str = "documentation/script/script";

/// File within the scripting documentation describing game interfaces and events.
const SCRIPTING_DOC_FILE: &str = "scripting_doc.html";

/// Folder of the vanilla scripts.
const VANILLA_SCRIPTS_FOLDER: &str = "script/";

/// Vanilla script declaring one table per event the game can trigger.
const VANILLA_EVENTS_SCRIPT: &str = "script/events.lua";

static TAG_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<[^>]*>").expect("valid regex"));
static ENTITY_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"&(#x[0-9a-fA-F]+|#[0-9]+|[a-zA-Z]+);").expect("valid regex"));
static INTERFACE_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[A-Z][A-Z0-9_]*_SCRIPT_INTERFACE").expect("valid regex"));
static IDENTIFIER_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[A-Za-z_][A-Za-z0-9_]*$").expect("valid regex"));
static SIGNATURE_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"(?s)<h3 class="function_name">(.*?)</h3>"#).expect("valid regex"));
static PARAMETER_TABLE_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"(?s)<table class="parameter_list">(.*?)</table>"#).expect("valid regex"));
static TABLE_ROW_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)<tr>(.*?)</tr>").expect("valid regex"));
static TABLE_CELL_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)<td>(.*?)</td>").expect("valid regex"));
static DB_TABLE_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<code>([a-z0-9_]+)</code>\s*(?:database\s+)?table\b").expect("valid regex"));
static RETURNS_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)<h4>Returns:</h4>\s*<ol>(.*?)</ol>").expect("valid regex"));
static LIST_ITEM_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)<li>(.*?)</li>").expect("valid regex"));
static CODE_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)<code>(.*?)</code>").expect("valid regex"));
static EVENT_ACCESSOR_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)Function Name: ([A-Za-z0-9_]+)</dd>\s*<dd>Interface: (.*?)</dd>(?:\s*<dd>Description: (.*?)</dd>)?").expect("valid regex"));
static DESCRIPTION_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"(?s)</dt>\s*<dd>(.*?)(?:<h4>|<p class="file_comment">|</dd>)"#).expect("valid regex"));
static OPTIONAL_DEFAULT_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)<i>\s*optional, default value=(.*?)</i>").expect("valid regex"));
static LINE_BREAK_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)<br\s*/?>").expect("valid regex"));
static INTERFACE_DESCRIPTION_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)<dd>Description: (.*?)</dd>").expect("valid regex"));
static INTERFACE_PARAMETERS_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)<dd>Parameters: (.*?)</dd>").expect("valid regex"));
static INTERFACE_FUNCTION_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"(?s)<dd>Function: <a name="[^"]*">([A-Za-z0-9_]+)</a></dd>(.*?)(?:<br>|$)"#).expect("valid regex"));
static INTERFACE_RETURN_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)<dd>Return: (.*?)</dd>").expect("valid regex"));
static EVENT_TABLE_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?m)^([A-Za-z0-9_]+)\s*=\s*\{\s*\}").expect("valid regex"));

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// Game environment in which a script runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum LuaEnvironment {
    Campaign,
    Battle,
    Frontend,
}

/// How a documented function is called.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LuaCallStyle {

    /// `function(...)`.
    Global,

    /// `owner:function(...)`.
    Method,

    /// `owner.function(...)`.
    Field,
}

/// Type of a value, as described by the docs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LuaType {

    /// The docs don't specify a usable type.
    Any,
    Nil,
    Boolean,
    Number,
    String,
    Table,
    Function,

    /// A game interface, like `FACTION_SCRIPT_INTERFACE`.
    Interface(String),

    /// A named object type, like `faction` or `battle_unit`. It may or may not match an interface or an owner.
    Object(String),
}

/// A parameter of a documented function.
#[derive(Clone, Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct LuaParameter {

    /// Name of the parameter, as written in the docs.
    name: String,

    /// Type of the parameter.
    lua_type: LuaType,

    /// If the parameter can be omitted.
    optional: bool,

    /// If the parameter takes any amount of values (`...`).
    variadic: bool,

    /// DB table, with the `_tables` suffix, whose keys this parameter expects.
    db_table: Option<String>,

    /// Description of the parameter.
    description: String,
}

/// A value returned by a documented function.
#[derive(Clone, Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct LuaReturn {

    /// Type of the value.
    lua_type: LuaType,

    /// Description of the value.
    description: String,
}

/// A context accessor of an event, like `context:faction()`.
#[derive(Clone, Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct LuaAccessor {

    /// Type of the value the accessor returns.
    lua_type: LuaType,

    /// Description of the accessor.
    description: String,
}

/// A documented function.
#[derive(Clone, Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct LuaFunction {

    /// Name of the function.
    name: String,

    /// How the function is called.
    call_style: LuaCallStyle,

    /// Signature of the function, as written in the docs.
    signature: String,

    /// Description of the function. Paragraphs are separated by line breaks.
    description: String,

    /// Parameters of the function. `None` if the docs don't describe them in a usable way.
    parameters: Option<Vec<LuaParameter>>,

    /// Returned values. Empty if the function returns nothing.
    returns: Vec<LuaReturn>,

    /// Environments whose docs include this function. Empty for interface methods, which are not tied to one.
    environments: BTreeSet<LuaEnvironment>,
}

/// Description of the Lua scripting API of a game.
#[derive(Clone, Debug, Default, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct LuaApi {

    /// Functions by owner, then by name. The owner is the part before `:` or `.` in a call
    /// (`cm`, `common`, `FACTION_SCRIPT_INTERFACE`, ...), and the empty string for global functions.
    owners: HashMap<String, HashMap<String, LuaFunction>>,

    /// Context accessors of each event, by event name, then by accessor name.
    events: HashMap<String, HashMap<String, LuaAccessor>>,

    /// Members and custom events defined by the vanilla scripts.
    script_definitions: LuaDefinitions,
}

/// Something in a script the docs describe, used to show its docs on hover.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum LuaHoverTarget {

    /// A documented function. Contains its owner (empty for globals) and its name.
    Function(String, String),

    /// A context accessor. Contains the event and the accessor.
    Accessor(String, String),

    /// An event.
    Event(String),
}

//---------------------------------------------------------------------------//
//                             Implementations
//---------------------------------------------------------------------------//

impl LuaEnvironment {

    /// Folder of the scripting docs holding the pages of this environment.
    pub fn docs_folder(&self) -> &'static str {
        match self {
            Self::Campaign => "campaign",
            Self::Battle => "battle",
            Self::Frontend => "frontend",
        }
    }
}

impl LuaType {

    /// This function returns the name of the type, for showing it to users.
    pub fn name(&self) -> &str {
        match self {
            Self::Any => "any",
            Self::Nil => "nil",
            Self::Boolean => "boolean",
            Self::Number => "number",
            Self::String => "string",
            Self::Table => "table",
            Self::Function => "function",
            Self::Interface(name) | Self::Object(name) => name,
        }
    }

    /// This function maps a type name, as written in the docs, to a type.
    ///
    /// # Arguments
    ///
    /// * `doc_name` - Type name from the docs, like `boolean`, `card32` or `FACTION_SCRIPT_INTERFACE`.
    ///
    /// # Returns
    ///
    /// The matching type, or [`LuaType::Any`] if the name doesn't describe one.
    pub fn from_doc_name(doc_name: &str) -> Self {
        let doc_name = doc_name.trim();
        if let Some(interface) = INTERFACE_REGEX.find(doc_name) {
            return Self::Interface(interface.as_str().to_owned());
        }

        let lower = doc_name.to_lowercase();
        let first_word = lower.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
            .find(|word| !word.is_empty())
            .unwrap_or_default();

        match first_word {
            "" | "nil" | "void" | "none" => Self::Nil,
            "bool" | "boolean" | "logical" => Self::Boolean,
            "number" | "integer" | "int" | "int32" | "card16" | "card32" | "float" | "float32" | "positive" | "index" | "distance" | "proportion" => Self::Number,
            "string" if lower.contains("table") => Self::Table,
            "string" => Self::String,
            "table" | "list" | "lists" | "lua" | "ca_std" | "sorted" | "accumulated" => Self::Table,
            "function" | "iterator" => Self::Function,
            "object" | "value" | "variable" | "userdata" | "address" | "any" => Self::Any,

            // Ranges, like "100 >= float >= 0".
            _ if first_word.starts_with(|c: char| c.is_ascii_digit()) => Self::Number,
            _ if lower.starts_with('(') => Self::Any,
            _ => Self::Object(first_word.to_owned()),
        }
    }
}

impl LuaApi {

    /// This function builds the API description from the scripting docs of an Assembly Kit.
    ///
    /// # Arguments
    ///
    /// * `docs_path` - Path of the scripting docs. See [`ASSEMBLY_KIT_SCRIPT_DOCS_PATH`].
    ///
    /// # Returns
    ///
    /// The API described by the docs.
    ///
    /// # Errors
    ///
    /// Returns [`RLibError::AssemblyKitNotFound`] if the docs folder doesn't exist, or an IO error if a page can't be read.
    pub fn from_assembly_kit(docs_path: &Path) -> Result<Self> {
        if !docs_path.is_dir() {
            return Err(RLibError::AssemblyKitNotFound);
        }

        let mut api = Self::default();
        for environment in [LuaEnvironment::Campaign, LuaEnvironment::Battle, LuaEnvironment::Frontend] {
            let folder = docs_path.join(environment.docs_folder());
            if !folder.is_dir() {
                continue;
            }

            for entry in fs::read_dir(&folder)? {
                let path = entry?.path();
                if path.extension().is_some_and(|extension| extension == "html") {
                    let page = fs::read(&path)?;
                    api.add_page(&String::from_utf8_lossy(&page), environment);
                }
            }
        }

        let scripting_doc_path = docs_path.join(SCRIPTING_DOC_FILE);
        if scripting_doc_path.is_file() {
            let scripting_doc = fs::read(&scripting_doc_path)?;
            api.add_scripting_doc(&String::from_utf8_lossy(&scripting_doc));
        }

        Ok(api)
    }

    /// This function completes the API with the vanilla scripts from the dependencies cache.
    ///
    /// Events from `script/events.lua` missing from the docs are added without accessors, and what the
    /// other vanilla scripts define is added to [`LuaApi::script_definitions`].
    ///
    /// # Arguments
    ///
    /// * `dependencies` - Dependencies cache with the vanilla files loaded.
    pub fn add_vanilla_scripts(&mut self, dependencies: &Dependencies) {
        let scripts = dependencies.files_by_path(&[ContainerPath::Folder(VANILLA_SCRIPTS_FOLDER.to_owned())], true, false, false)
            .into_par_iter()
            .filter(|(path, _)| path.ends_with(".lua"))
            .filter_map(|(path, file)| {

                // Vanilla files are loaded from disk on demand, so work on a copy to not need mutable access to the cache.
                let mut file = file.clone();
                file.load().ok()?;
                Some((path, String::from_utf8_lossy(file.cached().ok()?).to_string()))
            })
            .collect::<Vec<_>>();

        for (path, source) in &scripts {
            if path == VANILLA_EVENTS_SCRIPT {
                self.add_events_script(source);
            }
        }

        self.script_definitions = scripts.par_iter()
            .filter(|(path, _)| path != VANILLA_EVENTS_SCRIPT)
            .fold(LuaDefinitions::default, |mut definitions, (_, source)| {
                definitions.add_script(source);
                definitions
            })
            .reduce(LuaDefinitions::default, |mut definitions, other| {
                definitions.extend(other);
                definitions
            });
    }

    /// This function returns the docs of something in a script, formatted for a tooltip.
    ///
    /// # Arguments
    ///
    /// * `target` - What to return the docs of.
    ///
    /// # Returns
    ///
    /// The docs as Qt rich text, or `None` if the target is not documented.
    pub fn hover_html(&self, target: &LuaHoverTarget) -> Option<String> {
        match target {
            LuaHoverTarget::Function(owner, name) => {
                let function = self.function(owner, name)?;
                let mut html = format!("<p><code>{}</code></p>", html_escape(function.signature()));
                if !function.description().is_empty() {
                    html.push_str(&format!("<p>{}</p>", html_escape(function.description()).replace('\n', "<br/>")));
                }

                let parameters = function.parameters().as_deref().unwrap_or_default();
                if !parameters.is_empty() {
                    html.push_str("<p><b>Parameters:</b></p><ul>");
                    for parameter in parameters {
                        html.push_str(&format!("<li><code>{}</code>: {}</li>", html_escape(parameter.name()), html_escape(parameter.description())));
                    }
                    html.push_str("</ul>");
                }

                if !function.returns().is_empty() {
                    html.push_str("<p><b>Returns:</b></p><ul>");
                    for returned in function.returns() {
                        html.push_str(&format!("<li><code>{}</code> {}</li>", html_escape(returned.lua_type().name()), html_escape(returned.description())));
                    }
                    html.push_str("</ul>");
                }

                Some(html)
            }

            LuaHoverTarget::Accessor(event, name) => {
                let accessor = self.events.get(event)?.get(name)?;
                let mut html = format!("<p><code>{}:{}()</code> &rarr; <code>{}</code></p>", html_escape(event), html_escape(name), html_escape(accessor.lua_type().name()));
                if !accessor.description().is_empty() {
                    html.push_str(&format!("<p>{}</p>", html_escape(accessor.description())));
                }

                Some(html)
            }

            LuaHoverTarget::Event(event) => {
                let accessors = self.events.get(event)?;
                let mut html = format!("<p>Event <code>{}</code></p>", html_escape(event));
                if !accessors.is_empty() {
                    let mut names = accessors.keys().collect::<Vec<_>>();
                    names.sort();

                    html.push_str("<p><b>Context:</b></p><ul>");
                    for name in names {
                        let accessor = &accessors[name];
                        html.push_str(&format!("<li><code>context:{}()</code> &rarr; <code>{}</code> {}</li>", html_escape(name), html_escape(accessor.lua_type().name()), html_escape(accessor.description())));
                    }
                    html.push_str("</ul>");
                }

                Some(html)
            }
        }
    }

    /// This function returns a documented function.
    ///
    /// # Arguments
    ///
    /// * `owner` - Owner of the function. Empty for global functions.
    /// * `name` - Name of the function.
    ///
    /// # Returns
    ///
    /// The function, if it's documented.
    pub fn function(&self, owner: &str, name: &str) -> Option<&LuaFunction> {
        self.owners.get(owner).and_then(|functions| functions.get(name))
    }

    /// This function adds the functions of one per-page doc to the API.
    ///
    /// Functions already known from another environment's page only get the new environment added.
    ///
    /// # Arguments
    ///
    /// * `page` - HTML of the page.
    /// * `environment` - Environment whose docs the page belongs to.
    pub(crate) fn add_page(&mut self, page: &str, environment: LuaEnvironment) {
        for block in page.split(r#"<dl class="function">"#).skip(1) {
            let Some((owner, mut function)) = parse_function_block(block) else {
                continue;
            };

            let functions = self.owners.entry(owner).or_default();
            match functions.get_mut(&function.name) {
                Some(existing) => { existing.environments.insert(environment); },
                None => {
                    function.environments.insert(environment);
                    functions.insert(function.name.to_owned(), function);
                }
            }
        }
    }

    /// This function adds the events declared in `script/events.lua` that are not already known.
    ///
    /// # Arguments
    ///
    /// * `source` - Code of `script/events.lua`.
    pub(crate) fn add_events_script(&mut self, source: &str) {
        for captures in EVENT_TABLE_REGEX.captures_iter(source) {
            self.events.entry(captures[1].to_owned()).or_default();
        }
    }

    /// This function adds the game interfaces and events described in `scripting_doc.html` to the API.
    ///
    /// # Arguments
    ///
    /// * `scripting_doc` - HTML of `scripting_doc.html`.
    pub(crate) fn add_scripting_doc(&mut self, scripting_doc: &str) {
        let events_start = scripting_doc.find(">Event Functions<");
        let interfaces_start = scripting_doc.find(">Interface Functions<");

        if let Some(events_start) = events_start {
            let events_end = interfaces_start.filter(|end| *end > events_start).unwrap_or(scripting_doc.len());
            for block in scripting_doc[events_start..events_end].split(r#"<h4><a name=""#).skip(1) {
                let Some(event_name) = block.split('"').next() else {
                    continue;
                };

                let accessors = EVENT_ACCESSOR_REGEX.captures_iter(block)
                    .map(|captures| {
                        let accessor = LuaAccessor {
                            lua_type: LuaType::from_doc_name(&html_to_text(&captures[2])),
                            description: captures.get(3).map(|description| html_to_text(description.as_str())).unwrap_or_default(),
                        };

                        (captures[1].to_owned(), accessor)
                    })
                    .collect();

                self.events.insert(event_name.to_owned(), accessors);
            }
        }

        if let Some(interfaces_start) = interfaces_start {
            for block in scripting_doc[interfaces_start..].split(r#"<h4><a name=""#).skip(1) {
                let Some(interface_name) = block.split('"').next() else {
                    continue;
                };

                let functions = self.owners.entry(interface_name.to_owned()).or_default();
                for captures in INTERFACE_FUNCTION_REGEX.captures_iter(block) {
                    let name = &captures[1];
                    let details = &captures[2];
                    let returns = INTERFACE_RETURN_REGEX.captures(details)
                        .map(|return_captures| parse_returns([LuaReturn {
                            lua_type: LuaType::from_doc_name(&html_to_text(&return_captures[1])),
                            description: String::new(),
                        }]))
                        .unwrap_or_default();

                    // The parameters are either an example call, like `at_war_with(faction)`, or just the types, like `positive int`.
                    let parameters = INTERFACE_PARAMETERS_REGEX.captures(details).map(|parameters| html_to_text(&parameters[1])).unwrap_or_default();
                    let signature = if parameters.starts_with(&format!("{name}(")) {
                        format!("{interface_name}:{parameters}")
                    } else {
                        format!("{interface_name}:{name}({parameters})")
                    };

                    let function = LuaFunction {
                        name: name.to_owned(),
                        call_style: LuaCallStyle::Method,
                        signature,
                        description: INTERFACE_DESCRIPTION_REGEX.captures(details).map(|description| html_to_text(&description[1])).unwrap_or_default(),
                        parameters: None,
                        returns,
                        environments: BTreeSet::new(),
                    };

                    functions.insert(function.name.to_owned(), function);
                }
            }
        }
    }
}

//---------------------------------------------------------------------------//
//                              Parsing helpers
//---------------------------------------------------------------------------//

/// This function parses a `<dl class="function">` block of a per-page doc.
///
/// # Arguments
///
/// * `block` - HTML of the block, without the opening `<dl>` tag.
///
/// # Returns
///
/// The owner of the function and the function, or `None` if the block doesn't describe a function.
fn parse_function_block(block: &str) -> Option<(String, LuaFunction)> {
    if !block.contains(r#"name="function:"#) {
        return None;
    }

    let signature = tidy_signature(&html_to_text(&SIGNATURE_REGEX.captures(block)?[1]));
    let (head, arguments) = signature.split_once('(')?;
    let arguments = arguments.rsplit_once(')').map(|(arguments, _)| arguments).unwrap_or(arguments).to_owned();

    let head = head.trim();
    let (owner, name, call_style) = match head.rfind([':', '.']) {
        Some(position) if head[position..].starts_with(':') => (&head[..position], &head[position + 1..], LuaCallStyle::Method),
        Some(position) => (&head[..position], &head[position + 1..], LuaCallStyle::Field),
        None => ("", head, LuaCallStyle::Global),
    };

    if !IDENTIFIER_REGEX.is_match(name) || (!owner.is_empty() && !IDENTIFIER_REGEX.is_match(owner)) {
        return None;
    }

    let owner = owner.to_owned();
    let name = name.to_owned();

    let mut parameters = parse_parameters(&arguments);

    // The parameter table lists the parameters in signature order, starting at 1.
    if let Some(table) = PARAMETER_TABLE_REGEX.captures(block) {
        for row in TABLE_ROW_REGEX.captures_iter(&table[1]) {
            let cells = TABLE_CELL_REGEX.captures_iter(&row[1]).map(|cell| cell[1].to_owned()).collect::<Vec<_>>();
            let (Some(index), Some(description)) = (cells.first(), cells.get(2)) else {
                continue;
            };

            let Some(parameter) = html_to_text(index).parse::<usize>().ok().and_then(|index| parameters.get_mut(index.checked_sub(1)?)) else {
                continue;
            };

            match OPTIONAL_DEFAULT_REGEX.captures(description) {
                Some(default) => {
                    parameter.optional = true;
                    let rest = html_to_text(&OPTIONAL_DEFAULT_REGEX.replace(description, ""));
                    parameter.description = format!("(optional, default: {}) {rest}", html_to_text(&default[1])).trim_end().to_owned();
                }
                None => parameter.description = html_to_text(description),
            }

            if let Some(table_name) = DB_TABLE_REGEX.captures(description) {
                let table_name = &table_name[1];
                parameter.db_table = Some(if table_name.ends_with("_tables") { table_name.to_owned() } else { format!("{table_name}_tables") });
            }
        }
    }

    // Each returned value is its type in a `<code>` tag, followed by its description.
    let returns = RETURNS_REGEX.captures(block)
        .map(|returns| parse_returns(LIST_ITEM_REGEX.captures_iter(&returns[1])
            .map(|item| match CODE_REGEX.captures(&item[1]) {
                Some(code) => LuaReturn {
                    lua_type: LuaType::from_doc_name(&html_to_text(&code[1])),
                    description: html_to_text(&item[1][code.get(0).map_or(0, |code| code.end())..]),
                },
                None => LuaReturn { lua_type: LuaType::Any, description: html_to_text(&item[1]) },
            })))
        .unwrap_or_default();

    let description = DESCRIPTION_REGEX.captures(block)
        .map(|description| LINE_BREAK_REGEX.split(&description[1])
            .map(html_to_text)
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n"))
        .unwrap_or_default();

    let function = LuaFunction {
        name,
        call_style,
        signature,
        description,
        parameters: Some(parameters),
        returns,
        environments: BTreeSet::new(),
    };

    Some((owner, function))
}

/// This function parses the argument list of a signature, like `string key, [number amount], ... values`.
///
/// # Arguments
///
/// * `arguments` - Text between the parentheses of the signature.
///
/// # Returns
///
/// The parameters, in order.
fn parse_parameters(arguments: &str) -> Vec<LuaParameter> {
    arguments.split(',')
        .map(str::trim)
        .filter(|argument| !argument.is_empty())
        .map(|argument| {

            // Optional parameters are wrapped in brackets, which may be nested across several arguments.
            let optional = argument.contains('[');
            let argument = argument.trim_matches(|c: char| c == '[' || c == ']' || c.is_whitespace());
            let (doc_type, name) = argument.split_once(char::is_whitespace).unwrap_or((argument, ""));
            let variadic = doc_type == "...";

            LuaParameter {
                name: name.trim().to_owned(),
                lua_type: if variadic { LuaType::Any } else { LuaType::from_doc_name(doc_type) },
                optional,
                variadic,
                db_table: None,
                description: String::new(),
            }
        })
        .collect()
}

/// This function turns the documented returns of a function into its list of returned values.
///
/// # Arguments
///
/// * `returns` - Documented returns, in order.
///
/// # Returns
///
/// The returned values, without the `nil` entries used by the docs to mean "returns nothing".
fn parse_returns(returns: impl IntoIterator<Item = LuaReturn>) -> Vec<LuaReturn> {
    returns.into_iter().filter(|returned| returned.lua_type != LuaType::Nil).collect()
}

/// This function removes the spaces left around punctuation when turning a signature's markup into text.
///
/// # Arguments
///
/// * `signature` - Signature as text, like `cm:f( string key , [ number x ])`.
///
/// # Returns
///
/// The signature with normal spacing, like `cm:f(string key, [number x])`.
fn tidy_signature(signature: &str) -> String {
    signature.replace(" ,", ",")
        .replace("( ", "(")
        .replace(" )", ")")
        .replace("[ ", "[")
        .replace(" ]", "]")
}

/// This function escapes text so it can be embedded in HTML.
///
/// # Arguments
///
/// * `text` - Text to escape.
///
/// # Returns
///
/// The escaped text.
fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

/// This function turns a fragment of HTML into plain text.
///
/// # Arguments
///
/// * `html` - HTML fragment.
///
/// # Returns
///
/// The text of the fragment, with tags removed, entities decoded and whitespace collapsed.
fn html_to_text(html: &str) -> String {
    let text = TAG_REGEX.replace_all(html, " ");
    let text = ENTITY_REGEX.replace_all(&text, |captures: &regex::Captures| {
        let entity = &captures[1];
        let decoded = match entity {
            "amp" => Some('&'),
            "lt" => Some('<'),
            "gt" => Some('>'),
            "quot" => Some('"'),
            "apos" => Some('\''),
            "nbsp" | "emsp" | "ensp" | "thinsp" => Some(' '),
            _ => entity.strip_prefix("#x").map_or_else(
                || entity.strip_prefix('#').and_then(|code| code.parse::<u32>().ok()),
                |code| u32::from_str_radix(code, 16).ok()
            ).and_then(char::from_u32),
        };

        decoded.map_or_else(|| captures[0].to_owned(), String::from)
    });

    text.split_whitespace().collect::<Vec<_>>().join(" ")
}
