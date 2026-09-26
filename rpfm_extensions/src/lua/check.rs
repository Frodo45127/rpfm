//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Static checks of Lua scripts against the scripting API.
//!
//! Types are followed along call chains starting from known globals (`cm`, `core`, tables of functions like
//! `common`), from the context of event listeners registered with `core:add_listener`, and from locals
//! assigned from such chains. When a type can't be resolved, the chain is skipped instead of guessed, so
//! unresolved code never gets reported.

use full_moon::ast::{Ast, Call, Expression, FunctionArgs, FunctionBody, FunctionCall, Index, LocalAssignment, Assignment, FunctionDeclaration, Parameter, Prefix, Suffix, Var};
use full_moon::node::Node;
use full_moon::tokenizer::{Position, StringLiteralQuoteType, Symbol, TokenReference, TokenType};
use full_moon::visitors::Visitor;
use full_moon::LuaVersion;
use getset::Getters;

use std::collections::{HashMap, HashSet};

use super::{LuaApi, LuaCallStyle, LuaFunction, LuaHoverTarget, LuaType};

/// Globals holding objects whose methods come from several documented owners.
const GLOBAL_OWNERS: [(&str, &[&str]); 2] = [
    ("cm", &["cm", "campaign_manager"]),
    ("core", &["core"]),
];

/// Prefix of the events scripts trigger themselves. These are not part of the game's event list.
const SCRIPT_EVENT_PREFIX: &str = "ScriptEvent";

/// Methods used to trigger custom events.
const TRIGGER_EVENT_METHODS: [&str; 2] = ["trigger_event", "trigger_custom_event"];

/// Range in a script, as `((start line, start column), (end line, end column))`. All values are 0-based.
pub type LuaRange = ((u64, u64), (u64, u64));

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// A problem found in a script.
#[derive(Clone, Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct LuaIssue {

    /// What the problem is.
    kind: LuaIssueKind,

    /// Where the problem is.
    range: LuaRange,
}

/// Kinds of problems found in scripts.
#[derive(Clone, Debug, PartialEq)]
pub enum LuaIssueKind {

    /// The script can't be parsed. Contains the parser's message.
    SyntaxError(String),

    /// A method not documented for the type it's called on. Contains the type and the method.
    UnknownMethod(String, String),

    /// A documented function called with the wrong amount of arguments. Contains the function,
    /// the minimum and maximum (if any) amount of arguments it takes, and the amount passed.
    WrongArgumentCount(String, usize, Option<usize>, usize),

    /// A listener for an event that doesn't exist. Contains the event.
    UnknownEvent(String),
}

/// A string literal passed to a parameter that expects a key from a DB table.
#[derive(Clone, Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct LuaKeyReference {

    /// DB table, with the `_tables` suffix.
    table: String,

    /// The key used in the script.
    key: String,

    /// Where the key is.
    range: LuaRange,
}

/// A range of a script the docs describe.
#[derive(Clone, Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct LuaHover {

    /// What the docs describe. Use [`LuaApi::hover_html`] to get its docs.
    target: LuaHoverTarget,

    /// Where it is.
    range: LuaRange,
}

/// Results of checking a script.
#[derive(Clone, Debug, Default, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct LuaCheckResult {

    /// Problems found in the script.
    issues: Vec<LuaIssue>,

    /// DB keys used by the script, to be validated against the DB.
    key_references: Vec<LuaKeyReference>,

    /// Documented functions, accessors and events used by the script.
    hovers: Vec<LuaHover>,
}

/// Things scripts define themselves, which must not be reported as unknown.
#[derive(Clone, Debug, Default, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct LuaDefinitions {

    /// Members defined on globals, as `(owner, name)`, like `function cm:my_function()` or `cm.my_value = ...`.
    members: HashSet<(String, String)>,

    /// Custom events triggered by the scripts.
    events: HashSet<String>,
}

/// Type the checker has resolved for a value.
#[derive(Clone, Debug, PartialEq)]
enum ResolvedType {

    /// An object whose methods are documented under these owners, and whose own name is the first of them.
    Owners(Vec<String>),

    /// The context received by listeners of an event.
    EventContext(String),
}

/// Visitor doing the checks.
struct Checker<'a> {
    api: &'a LuaApi,
    definitions: &'a LuaDefinitions,

    /// Known types of variables, one map per function. `None` means the variable exists but its type is unknown.
    scopes: Vec<HashMap<String, Option<ResolvedType>>>,

    /// Types of the first parameter of listener functions, by the start byte of their body.
    listener_contexts: HashMap<usize, ResolvedType>,

    result: LuaCheckResult,
}

/// Visitor collecting the definitions of a script.
struct DefinitionsCollector<'a> {
    definitions: &'a mut LuaDefinitions,
}

/// An argument of a call.
#[derive(Clone, Copy)]
enum Argument<'a> {
    Expression(&'a Expression),

    /// String passed without parentheses, like `f "text"`.
    String(&'a TokenReference),

    /// Table passed without parentheses, like `f { ... }`.
    Table,
}

//---------------------------------------------------------------------------//
//                             Implementations
//---------------------------------------------------------------------------//

impl LuaDefinitions {

    /// This function adds the definitions of a script.
    ///
    /// Scripts that can't be parsed are ignored.
    ///
    /// # Arguments
    ///
    /// * `source` - Code of the script.
    pub fn add_script(&mut self, source: &str) {
        if let Ok(ast) = full_moon::parse_fallible(source, LuaVersion::lua51()).into_result() {
            DefinitionsCollector { definitions: self }.visit_ast(&ast);
        }
    }

    /// This function adds the definitions of another set to this one.
    ///
    /// # Arguments
    ///
    /// * `other` - Definitions to add.
    pub fn extend(&mut self, other: Self) {
        self.members.extend(other.members);
        self.events.extend(other.events);
    }
}

/// This function checks a script.
///
/// # Arguments
///
/// * `source` - Code of the script.
/// * `api` - API of the game. Without it, only the syntax is checked.
/// * `definitions` - Definitions from all the scripts the checked one can see, including itself.
///
/// # Returns
///
/// The problems and DB keys found in the script.
pub fn check_script(source: &str, api: Option<&LuaApi>, definitions: &LuaDefinitions) -> LuaCheckResult {
    let ast = match full_moon::parse_fallible(source, LuaVersion::lua51()).into_result() {
        Ok(ast) => ast,
        Err(errors) => {
            let issues = errors.iter()
                .map(|error| LuaIssue {
                    kind: LuaIssueKind::SyntaxError(error.error_message().to_string()),
                    range: (position(error.range().0), position(error.range().1)),
                })
                .collect();

            return LuaCheckResult { issues, ..Default::default() };
        }
    };

    match api {
        Some(api) => check_ast(&ast, api, definitions),
        None => LuaCheckResult::default(),
    }
}

/// This function runs the API checks over a parsed script.
///
/// # Arguments
///
/// * `ast` - The parsed script.
/// * `api` - API of the game.
/// * `definitions` - Definitions from all the scripts the checked one can see.
///
/// # Returns
///
/// The problems and DB keys found in the script.
fn check_ast(ast: &Ast, api: &LuaApi, definitions: &LuaDefinitions) -> LuaCheckResult {
    let mut checker = Checker {
        api,
        definitions,
        scopes: vec![HashMap::new()],
        listener_contexts: HashMap::new(),
        result: LuaCheckResult::default(),
    };

    checker.visit_ast(ast);
    checker.result
}

impl Checker<'_> {

    /// This function returns the type of a variable, looking from the innermost function outwards, then at the globals.
    fn variable_type(&self, name: &str) -> Option<ResolvedType> {
        if let Some(scope) = self.scopes.iter().rev().find(|scope| scope.contains_key(name)) {
            return scope.get(name).cloned().flatten();
        }

        if let Some((_, owners)) = GLOBAL_OWNERS.iter().find(|(global, _)| *global == name) {
            return Some(ResolvedType::Owners(owners.iter().map(|owner| owner.to_string()).collect()));
        }

        // Owners with only `.` functions are global tables of functions, like `common`. Other owners in the
        // docs are named after example variables (`unit`, `army`, ...), which don't exist as globals.
        self.api.owners().get(name)
            .filter(|functions| !functions.is_empty() && functions.values().all(|function| *function.call_style() == LuaCallStyle::Field))
            .map(|_| ResolvedType::Owners(vec![name.to_owned()]))
    }

    /// This function turns a documented type into a type the checker can follow.
    fn resolve(&self, lua_type: &LuaType) -> Option<ResolvedType> {
        let owner = match lua_type {
            LuaType::Interface(interface) => interface.to_owned(),
            LuaType::Object(object) => {
                let interface = format!("{}_SCRIPT_INTERFACE", object.to_uppercase());
                if self.api.owners().contains_key(&interface) {
                    interface
                } else {
                    object.to_owned()
                }
            }
            _ => return None,
        };

        if self.api.owners().contains_key(&owner) {
            Some(ResolvedType::Owners(vec![owner]))
        } else {
            None
        }
    }

    /// This function follows a call chain, reporting problems found on it if `report` is true.
    ///
    /// # Returns
    ///
    /// The type of the value the chain evaluates to, if it can be resolved.
    fn follow_chain<'s>(&mut self, prefix: &Prefix, suffixes: impl Iterator<Item = &'s Suffix>, report: bool) -> Option<ResolvedType> {
        let Prefix::Name(root) = prefix else {
            return None;
        };

        let root_name = root.token().to_string();
        let suffixes = suffixes.collect::<Vec<_>>();

        let mut current = match self.variable_type(&root_name) {
            Some(current) => current,

            // Calls to global functions only get their arguments checked.
            None => {
                let is_local = self.scopes.iter().any(|scope| scope.contains_key(&root_name));
                let global = if is_local { None } else { self.api.function("", &root_name) };
                return match (global, suffixes.first()) {
                    (Some(function), Some(Suffix::Call(Call::AnonymousCall(args)))) => {
                        if report {
                            self.push_hover(LuaHoverTarget::Function(String::new(), root_name.to_owned()), root);
                            if suffixes.len() == 1 {
                                self.check_call(function, &root_name, args, root);
                            }
                        }
                        None
                    }
                    _ => None,
                };
            }
        };

        let mut index = 0;
        while index < suffixes.len() {
            let (name_token, args, via_dot) = match (suffixes[index], suffixes.get(index + 1)) {
                (Suffix::Call(Call::MethodCall(method_call)), _) => (method_call.name(), method_call.args(), false),
                (Suffix::Index(Index::Dot { name, .. }), Some(Suffix::Call(Call::AnonymousCall(args)))) => {
                    index += 1;
                    (name, args, true)
                },
                _ => return None,
            };

            let name = name_token.token().to_string();
            current = match current {
                ResolvedType::EventContext(event) => {
                    let accessors = self.api.events().get(&event)?;
                    match accessors.get(&name) {
                        Some(accessor) => {
                            if report {
                                self.push_hover(LuaHoverTarget::Accessor(event.to_owned(), name.to_owned()), name_token);
                            }
                            self.resolve(accessor.lua_type())?
                        }
                        None => {
                            if report && !via_dot && !accessors.is_empty() {
                                self.push_issue(LuaIssueKind::UnknownMethod(format!("{event} context"), name), name_token);
                            }
                            return None;
                        }
                    }
                }

                ResolvedType::Owners(owners) => {
                    let function = owners.iter().find_map(|owner| self.api.function(owner, &name).map(|function| (owner, function)));
                    match function {
                        Some((owner, function)) => {
                            if report {
                                self.push_hover(LuaHoverTarget::Function(owner.to_owned(), name.to_owned()), name_token);
                            }

                            // Calls using the other syntax pass `self` differently, so their arguments can't be counted.
                            let expected_via_dot = *function.call_style() == LuaCallStyle::Field;
                            if report && via_dot == expected_via_dot {
                                self.check_call(function, &name, args, name_token);
                            }

                            self.resolve(function.returns().first()?.lua_type())?
                        }
                        None => {

                            // Without docs for the object there is nothing to compare against.
                            if !owners.iter().any(|owner| self.api.owners().contains_key(owner)) {
                                return None;
                            }

                            let defined = owners.iter()
                                .chain(std::iter::once(&root_name))
                                .any(|owner| {
                                    let member = (owner.to_owned(), name.to_owned());
                                    self.definitions.members.contains(&member) || self.api.script_definitions().members.contains(&member)
                                });

                            if report && !via_dot && !defined {
                                self.push_issue(LuaIssueKind::UnknownMethod(owners[0].to_owned(), name), name_token);
                            }
                            return None;
                        }
                    }
                }
            };

            index += 1;
        }

        Some(current)
    }

    /// This function checks the arguments passed to a documented function, and collects the DB keys passed to it.
    fn check_call(&mut self, function: &LuaFunction, name: &str, args: &FunctionArgs, name_token: &TokenReference) {
        let Some(parameters) = function.parameters() else {
            return;
        };

        let arguments = call_arguments(args);
        let minimum = parameters.iter().rposition(|parameter| !parameter.optional() && !parameter.variadic()).map_or(0, |position| position + 1);
        let maximum = if parameters.iter().any(|parameter| *parameter.variadic()) { None } else { Some(parameters.len()) };

        // A call or `...` as last argument can expand to any amount of values.
        let open_ended = match arguments.last() {
            Some(Argument::Expression(Expression::FunctionCall(_))) => true,
            Some(Argument::Expression(Expression::Symbol(symbol))) => matches!(symbol.token_type(), TokenType::Symbol { symbol: Symbol::Ellipsis }),
            _ => false,
        };

        let too_few = !open_ended && arguments.len() < minimum;
        let too_many = maximum.is_some_and(|maximum| arguments.len() > maximum);
        if too_few || too_many {
            self.push_issue(LuaIssueKind::WrongArgumentCount(name.to_owned(), minimum, maximum, arguments.len()), name_token);
        }

        for (parameter, argument) in parameters.iter().zip(arguments.iter()) {
            if *parameter.variadic() {
                break;
            }

            // Empty strings are how many functions take "no key", so they're never a reference.
            if let (Some(table), Some((key, token))) = (parameter.db_table(), string_literal(argument)) {
                if key.is_empty() {
                    continue;
                }

                if let Some(range) = string_literal_range(token) {
                    self.result.key_references.push(LuaKeyReference { table: table.to_owned(), key, range });
                }
            }
        }
    }

    /// This function checks the event of a `core:add_listener` call, and types the context received by its functions.
    fn check_listener(&mut self, call: &FunctionCall) {
        let Prefix::Name(root) = call.prefix() else {
            return;
        };

        let suffixes = call.suffixes().collect::<Vec<_>>();
        let [Suffix::Call(Call::MethodCall(method_call))] = suffixes.as_slice() else {
            return;
        };

        if root.token().to_string() != "core" || method_call.name().token().to_string() != "add_listener" || self.variable_type("core").is_none() {
            return;
        }

        let arguments = call_arguments(method_call.args());
        let Some((event, event_token)) = arguments.get(1).and_then(string_literal) else {
            return;
        };

        if self.api.events().contains_key(&event) {
            if let Some(range) = string_literal_range(event_token) {
                self.result.hovers.push(LuaHover { target: LuaHoverTarget::Event(event.to_owned()), range });
            }
        } else {
            let defined = self.definitions.events.contains(&event) || self.api.script_definitions().events.contains(&event);
            if !event.starts_with(SCRIPT_EVENT_PREFIX) && !defined {
                self.push_issue(LuaIssueKind::UnknownEvent(event), event_token);
            }
            return;
        }

        for argument in arguments.iter().skip(2).take(2) {
            if let Argument::Expression(Expression::Function(function)) = argument {
                if let Some(start) = Node::start_position(function.body()) {
                    self.listener_contexts.insert(start.bytes(), ResolvedType::EventContext(event.to_owned()));
                }
            }
        }
    }

    fn push_hover(&mut self, target: LuaHoverTarget, node: &impl Node) {
        if let Some(range) = node_range(node) {
            self.result.hovers.push(LuaHover { target, range });
        }
    }

    fn push_issue(&mut self, kind: LuaIssueKind, node: &impl Node) {
        if let Some(range) = node_range(node) {
            self.result.issues.push(LuaIssue { kind, range });
        }
    }
}

impl Visitor for Checker<'_> {
    fn visit_function_body(&mut self, body: &FunctionBody) {
        let mut scope = HashMap::new();
        let context = Node::start_position(body).and_then(|start| self.listener_contexts.remove(&start.bytes()));
        for (index, parameter) in body.parameters().iter().enumerate() {
            if let Parameter::Name(name) = parameter {
                let parameter_type = if index == 0 { context.clone() } else { None };
                scope.insert(name.token().to_string(), parameter_type);
            }
        }

        self.scopes.push(scope);
    }

    fn visit_function_body_end(&mut self, _body: &FunctionBody) {
        self.scopes.pop();
    }

    fn visit_function_call(&mut self, call: &FunctionCall) {
        self.check_listener(call);
        self.follow_chain(call.prefix(), call.suffixes(), true);
    }

    fn visit_local_assignment_end(&mut self, assignment: &LocalAssignment) {
        let values = assignment.expressions().iter().collect::<Vec<_>>();
        for (index, name) in assignment.names().iter().enumerate() {
            let value_type = match values.get(index) {
                Some(Expression::FunctionCall(call)) => self.follow_chain(call.prefix(), call.suffixes(), false),
                Some(Expression::Var(Var::Name(other))) => self.variable_type(&other.token().to_string()),
                _ => None,
            };

            if let Some(scope) = self.scopes.last_mut() {
                scope.insert(name.token().to_string(), value_type);
            }
        }
    }

    fn visit_assignment(&mut self, assignment: &Assignment) {
        for variable in assignment.variables() {
            if let Var::Name(name) = variable {
                let name = name.token().to_string();
                if let Some(scope) = self.scopes.iter_mut().rev().find(|scope| scope.contains_key(&name)) {
                    scope.insert(name, None);
                }
            }
        }
    }
}

impl Visitor for DefinitionsCollector<'_> {
    fn visit_function_declaration(&mut self, declaration: &FunctionDeclaration) {
        let names = declaration.name().names().iter().map(|name| name.token().to_string()).collect::<Vec<_>>();
        let member = match declaration.name().method_name() {
            Some(method) => names.last().map(|owner| (owner.to_owned(), method.token().to_string())),
            None if names.len() >= 2 => Some((names[names.len() - 2].to_owned(), names[names.len() - 1].to_owned())),
            None => None,
        };

        if let Some(member) = member {
            self.definitions.members.insert(member);
        }
    }

    fn visit_assignment(&mut self, assignment: &Assignment) {
        for variable in assignment.variables() {
            if let Var::Expression(expression) = variable {
                let suffixes = expression.suffixes().collect::<Vec<_>>();
                if let (Prefix::Name(owner), [Suffix::Index(Index::Dot { name, .. })]) = (expression.prefix(), suffixes.as_slice()) {
                    self.definitions.members.insert((owner.token().to_string(), name.token().to_string()));
                }
            }
        }
    }

    fn visit_method_call(&mut self, method_call: &full_moon::ast::MethodCall) {
        if TRIGGER_EVENT_METHODS.contains(&method_call.name().token().to_string().as_str()) {
            if let Some((event, _)) = call_arguments(method_call.args()).first().and_then(string_literal) {
                self.definitions.events.insert(event);
            }
        }
    }
}

//---------------------------------------------------------------------------//
//                                 Helpers
//---------------------------------------------------------------------------//

/// This function returns the arguments of a call.
fn call_arguments(args: &FunctionArgs) -> Vec<Argument<'_>> {
    match args {
        FunctionArgs::Parentheses { arguments, .. } => arguments.iter().map(Argument::Expression).collect(),
        FunctionArgs::String(string) => vec![Argument::String(string)],
        FunctionArgs::TableConstructor(_) => vec![Argument::Table],
        _ => vec![],
    }
}

/// This function returns the range of the text of a string literal, without its quotes.
fn string_literal_range(token: &TokenReference) -> Option<LuaRange> {
    let ((start_line, start_column), (end_line, end_column)) = node_range(token)?;
    match token.token_type() {
        TokenType::StringLiteral { quote_type: StringLiteralQuoteType::Double | StringLiteralQuoteType::Single, .. } =>
            Some(((start_line, start_column + 1), (end_line, end_column.saturating_sub(1)))),
        _ => Some(((start_line, start_column), (end_line, end_column))),
    }
}

/// This function returns the value and token of an argument, if it's a string literal.
fn string_literal<'a>(argument: &Argument<'a>) -> Option<(String, &'a TokenReference)> {
    let token = match *argument {
        Argument::Expression(Expression::String(token)) | Argument::String(token) => token,
        _ => return None,
    };

    match token.token_type() {
        TokenType::StringLiteral { literal, .. } => Some((literal.to_string(), token)),
        _ => None,
    }
}

/// This function returns the range of a node.
fn node_range(node: &impl Node) -> Option<LuaRange> {
    Some((position(Node::start_position(node)?), position(Node::end_position(node)?)))
}

/// This function turns a parser position (1-based) into a 0-based `(line, column)`.
fn position(position: Position) -> (u64, u64) {
    (position.line().saturating_sub(1) as u64, position.character().saturating_sub(1) as u64)
}
