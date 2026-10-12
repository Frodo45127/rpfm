//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Generates the method pages of the server docs from the JSON schemas of the API types, and checks
//! the ones in `docs/server` are up to date.
//!
//! To update them after changing the API, run `RPFM_UPDATE_DOCS=1 cargo test -p rpfm_ipc method_docs`.

use schemars::generate::{Contract, SchemaGenerator, SchemaSettings};
use schemars::JsonSchema;
use serde_json::{Map, Value};

use std::any::type_name;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

use super::*;

/// Environment variable that makes the test write the pages instead of checking them.
const UPDATE_DOCS_VAR: &str = "RPFM_UPDATE_DOCS";

/// Notice at the top of every generated page.
const GENERATED_NOTICE: &str = "<!-- Generated from the API types by `rpfm_ipc/src/api/method_docs.rs`. Don't edit it by hand. -->";

/// Introduction of the methods index page.
const INDEX_INTRO: &str = "\
Every method of the API, grouped by domain. Each domain's page lists the params and the result of its methods, \
generated from the request and response structs of `rpfm_ipc::api`. Methods marked as jobs answer right away \
with a job ID, and send their result when the job ends (see [Jobs](./ws-protocol.md#jobs)).

Methods meant for clients with their own views of the files, like the UI, return RPFM's own types (decoded files, \
typed search matches, full diagnostics), shown as `any` in their pages. Simpler clients should prefer the paginated \
methods next to them, like `table.rows`, `search.matches` or `diagnostics.list`.";

/// A method of a page, with the schemas of its request and response.
struct Method {
    name: &'static str,
    request_type: &'static str,
    request: Value,
    response_type: &'static str,

    /// `None` if the response is one of RPFM's own types, without a schema.
    response: Option<Value>,
    is_job: bool,
}

/// A page of the docs, with the methods of a domain.
struct Page {
    title: &'static str,
    file_stem: &'static str,

    /// Source of the module the page is about, for its introduction and to find its methods.
    source: &'static str,
    methods: Vec<Method>,

    /// Generators of the requests, with the rules to deserialize them, and of the responses, with the rules to serialize them.
    requests: SchemaGenerator,
    responses: SchemaGenerator,
}

impl Page {
    fn new(title: &'static str, file_stem: &'static str, source: &'static str) -> Self {
        Self {
            title,
            file_stem,
            source,
            methods: vec![],
            requests: generator(Contract::Deserialize),
            responses: generator(Contract::Serialize),
        }
    }

    fn method<R: Request + JsonSchema>(&mut self) where R::Response: JsonSchema {
        let response = self.responses.subschema_for::<R::Response>().to_value();
        self.add_method::<R>(Some(response));
    }

    /// Adds a method whose response is one of RPFM's own types, which have no schema.
    fn method_with_opaque_response<R: Request + JsonSchema>(&mut self) {
        self.add_method::<R>(None);
    }

    fn add_method<R: Request + JsonSchema>(&mut self, response: Option<Value>) {
        self.methods.push(Method {
            name: R::METHOD,
            request_type: short_type_name::<R>(),
            request: self.requests.subschema_for::<R>().to_value(),
            response_type: short_type_name::<R::Response>(),
            response,
            is_job: R::IS_JOB,
        });
    }

    /// Path of the page, relative to `docs/server`.
    fn path(&self) -> String {
        format!("methods/{}.md", self.file_stem)
    }

    /// Introduction of the page: the docs of its module.
    fn intro(&self) -> String {
        let intro = self.source.lines()
            .take_while(|line| line.starts_with("//!") || line.starts_with("//") || line.is_empty())
            .filter_map(|line| line.strip_prefix("//!"))
            .map(str::trim)
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .to_owned();

        without_rustdoc_links(&intro)
    }

    /// Types defined by the requests and responses of the page, by name.
    fn definitions(&self) -> BTreeMap<&String, &Value> {
        let mut definitions = BTreeMap::new();
        definitions.extend(self.responses.definitions());
        definitions.extend(self.requests.definitions());
        definitions
    }

    fn render(&self, orders: &FieldOrders) -> String {
        let definitions = self.definitions();
        let mut referenced = BTreeSet::new();
        let mut text = format!("{GENERATED_NOTICE}\n\n# {}\n\n{}\n", self.title, self.intro());

        for method in &self.methods {
            text.push_str(&format!("\n## `{}`\n\n", method.name));

            let request = resolve(&method.request, &definitions);
            text.push_str(&format!("{}\n\n", method_description(method.name, request)));

            if method.is_job {
                text.push_str("Runs as a job: answers right away with a job ID, and sends its result when the job ends (see [Jobs](../ws-protocol.md#jobs)).\n\n");
            }

            text.push_str(&format!("**Params** (`{}`)\n\n{}\n", method.request_type, fields_or_type(request, orders.get(method.request_type), &mut referenced)));
            text.push_str(&format!("**Result** (`{}`)\n\n", method.response_type));
            match &method.response {
                Some(response) => text.push_str(&fields_or_type(resolve(response, &definitions), orders.get(method.response_type), &mut referenced)),
                None => text.push_str("One of RPFM's own types, without a schema. See its docs in `rpfm_lib` or `rpfm_extensions`.\n"),
            }
        }

        // Types are rendered until there are no new ones, as they can reference other types.
        let mut rendered = BTreeSet::new();
        let mut types = String::new();
        while let Some(name) = referenced.iter().find(|name| !rendered.contains(*name)).cloned() {
            if let Some(schema) = definitions.get(&name) {
                types.push_str(&format!("\n### Type `{name}`\n\n{}", definition(schema, orders.get(&name), &mut referenced)));
            }
            rendered.insert(name);
        }

        if !types.is_empty() {
            text.push_str(&format!("\n## Types\n{types}"));
        }

        text
    }
}

/// Returns a schema generator following the provided serde contract.
fn generator(contract: Contract) -> SchemaGenerator {
    let mut settings = SchemaSettings::draft2020_12();
    settings.contract = contract;
    settings.into_generator()
}

/// Returns the name of a type without its path or generics.
fn short_type_name<T: ?Sized>() -> &'static str {
    let name = type_name::<T>();
    let name = name.split('<').next().unwrap_or(name);
    name.rsplit("::").next().unwrap_or(name)
}

/// Returns the schema a `$ref` points to, or the schema itself if it isn't one.
fn resolve<'a>(schema: &'a Value, definitions: &BTreeMap<&String, &'a Value>) -> &'a Value {
    schema.get("$ref")
        .and_then(Value::as_str)
        .and_then(|reference| reference.strip_prefix("#/$defs/"))
        .and_then(|name| definitions.get(&name.to_owned()).copied())
        .unwrap_or(schema)
}

/// Returns the description of a method, without the `method.name: ` prefix of its request's docs.
fn method_description(method: &str, request: &Value) -> String {
    let full_description = description(request).replace(" Runs as a job.", "");
    let description = full_description.strip_prefix(&format!("`{method}`: ")).unwrap_or(&full_description);
    let mut chars = description.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

fn description(schema: &Value) -> String {
    without_rustdoc_links(schema.get("description").and_then(Value::as_str).unwrap_or_default())
}

/// Returns a text from the Rust docs with its links to Rust items, like [`JobStarted`], as plain code.
fn without_rustdoc_links(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find("[`") {
        result.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        match after.find("`]") {
            Some(end) => {
                result.push_str(&after[..end + 1]);
                rest = &after[end + 2..];

                // Drop the target of explicit links, like `(super::Item)`.
                if rest.starts_with('(') {
                    rest = rest.find(')').map_or(rest, |close| &rest[close + 1..]);
                }
            }
            None => {
                result.push('[');
                rest = after;
            }
        }
    }

    result.push_str(rest);
    result
}

/// Returns a description in a single line, to fit in a table cell.
fn table_cell(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ").replace('|', "\\|")
}

/// Returns the anchor of a type in its page.
fn type_anchor(name: &str) -> String {
    format!("type-{}", name.to_lowercase())
}

/// Returns the fields table of an object schema, or its type if it's something else.
fn fields_or_type(schema: &Value, order: Option<&Vec<String>>, referenced: &mut BTreeSet<String>) -> String {
    let variants = schema.get("oneOf").or_else(|| schema.get("anyOf")).and_then(Value::as_array);
    let mut text = match schema.get("properties").and_then(Value::as_object) {
        Some(properties) if !properties.is_empty() => fields_table(&sorted_properties(properties, order), &required(schema), referenced),
        _ if variants.is_some() => String::new(),
        _ if schema.get("type").and_then(Value::as_str) == Some("object") && schema.get("additionalProperties").is_none_or(|values| values == false) => "Empty object: `{}`.\n".to_owned(),
        _ => return format!("{}.\n", type_text(schema, referenced)),
    };

    // Enums flattened into a struct add their variants to its fields.
    if let Some(variants) = variants {
        let intro = if text.is_empty() { "One of:\n\n" } else { "\nPlus one of:\n\n" };
        text.push_str(intro);
        for variant in variants {
            text.push_str(&variant_text(variant, referenced));
        }
    }

    text
}

/// Returns the properties of an object schema in the order their fields are declared, or by name if it's unknown.
fn sorted_properties<'a>(properties: &'a Map<String, Value>, order: Option<&Vec<String>>) -> Vec<(&'a String, &'a Value)> {
    let mut sorted = properties.iter().collect::<Vec<_>>();
    if let Some(order) = order {
        sorted.sort_by_key(|(name, _)| order.iter().position(|field| field == *name).unwrap_or(usize::MAX));
    }

    sorted
}

/// Returns the names of the required properties of an object schema.
fn required(schema: &Value) -> BTreeSet<String> {
    schema.get("required")
        .and_then(Value::as_array)
        .map(|names| names.iter().filter_map(Value::as_str).map(str::to_owned).collect())
        .unwrap_or_default()
}

fn fields_table(properties: &[(&String, &Value)], required: &BTreeSet<String>, referenced: &mut BTreeSet<String>) -> String {
    let mut table = "| Name | Type | Required | Description |\n|------|------|----------|-------------|\n".to_owned();
    for (name, schema) in properties {
        table.push_str(&format!("| `{name}` | {} | {} | {} |\n", type_text(schema, referenced), required_text(name, schema, required), table_cell(&description(schema))));
    }

    table
}

/// Returns if a field is required, with its default value if it has a meaningful one.
fn required_text(name: &str, schema: &Value, required: &BTreeSet<String>) -> String {
    if required.contains(name) {
        return "yes".to_owned();
    }

    match schema.get("default") {
        Some(Value::Null) | None => "no".to_owned(),
        Some(default) => format!("no, default `{default}`"),
    }
}

/// Returns a short text with the type of a schema, linking the types it references.
fn type_text(schema: &Value, referenced: &mut BTreeSet<String>) -> String {
    let Some(object) = schema.as_object() else {
        return "`any`".to_owned();
    };

    if let Some(name) = object.get("$ref").and_then(Value::as_str).and_then(|reference| reference.strip_prefix("#/$defs/")) {
        referenced.insert(name.to_owned());
        return format!("[`{name}`](#{})", type_anchor(name));
    }

    if let Some(value) = object.get("const") {
        return format!("`{value}`");
    }

    if let Some(values) = object.get("enum").and_then(Value::as_array) {
        return values.iter().map(|value| format!("`{value}`")).collect::<Vec<_>>().join(" or ");
    }

    for key in ["oneOf", "anyOf"] {
        if let Some(variants) = object.get(key).and_then(Value::as_array) {
            return variants.iter().map(|variant| type_text(variant, referenced)).collect::<Vec<_>>().join(" or ");
        }
    }

    match object.get("type") {
        Some(Value::Array(types)) => types.iter()
            .filter_map(Value::as_str)
            .map(|kind| single_type_text(kind, object, referenced))
            .collect::<Vec<_>>()
            .join(" or "),
        Some(Value::String(kind)) => single_type_text(kind, object, referenced),
        _ => "`any`".to_owned(),
    }
}

/// Returns the text of one of the types of a schema, like `array of string`.
fn single_type_text(kind: &str, schema: &Map<String, Value>, referenced: &mut BTreeSet<String>) -> String {
    match kind {
        "array" => match (schema.get("items"), schema.get("prefixItems").and_then(Value::as_array)) {
            (Some(items), _) => format!("array of {}", type_text(items, referenced)),
            (None, Some(items)) => format!("[{}]", items.iter().map(|item| type_text(item, referenced)).collect::<Vec<_>>().join(", ")),
            (None, None) => "`array`".to_owned(),
        },
        "object" => {
            let pattern_values = schema.get("patternProperties").and_then(Value::as_object).and_then(|patterns| patterns.values().next());
            match pattern_values.or_else(|| schema.get("additionalProperties")) {
                Some(Value::Bool(false)) | None => "`object`".to_owned(),
                Some(values) => format!("map of {}", type_text(values, referenced)),
            }
        },
        kind => format!("`{kind}`"),
    }
}

/// Returns the description and the shape of a type of the Types section.
fn definition(schema: &Value, order: Option<&Vec<String>>, referenced: &mut BTreeSet<String>) -> String {
    let mut text = String::new();
    let type_description = description(schema);
    if !type_description.is_empty() {
        text.push_str(&format!("{type_description}\n\n"));
    }

    text.push_str(&fields_or_type(schema, order, referenced));
    text
}

/// Returns a variant of an enum as a list item, with its fields as nested items.
fn variant_text(variant: &Value, referenced: &mut BTreeSet<String>) -> String {
    let variant_description = table_cell(&description(variant));
    let Some(properties) = variant.get("properties").and_then(Value::as_object) else {
        return format!("- {}: {variant_description}\n", type_text(variant, referenced));
    };

    // Properties with a constant value are tags identifying the variant, so they go first, and in its shape instead of its fields.
    let required = required(variant);
    let mut properties = properties.iter().collect::<Vec<_>>();
    properties.sort_by_key(|(name, schema)| (schema.get("const").is_none(), !required.contains(*name)));

    let shape = properties.iter()
        .map(|(name, schema)| match schema.get("const") {
            Some(value) => format!("\"{name}\": {value}"),
            None if required.contains(*name) => format!("\"{name}\": …"),
            None => format!("\"{name}\"?: …"),
        })
        .collect::<Vec<_>>()
        .join(", ");

    let mut text = format!("- `{{ {shape} }}`: {variant_description}\n");
    for (name, schema) in properties.into_iter().filter(|(_, schema)| schema.get("const").is_none()) {
        let field_description = table_cell(&description(schema));
        let separator = if field_description.is_empty() { "" } else { ": " };
        text.push_str(&format!("    - `{name}` ({}, {}){separator}{field_description}\n", type_text(schema, referenced), required_text(name, schema, &required)));
    }

    text
}

/// Declared order of the fields of the structs of the API, by the name of their schema.
type FieldOrders = BTreeMap<String, Vec<String>>;

/// Returns the order the fields of each struct are declared in, reading the sources of the API modules.
///
/// Schemas keep their properties sorted by name, so this is needed to show them in the order they're written.
fn field_orders(pages: &[Page]) -> FieldOrders {
    let sources = pages.iter().map(|page| page.source).chain([include_str!("mod.rs")]);
    let mut orders = FieldOrders::new();
    for source in sources {
        let mut renamed = None;
        let mut current: Option<(String, Vec<String>)> = None;
        for line in source.lines() {
            if let Some((name, fields)) = current.as_mut() {
                if line == "}" {
                    orders.insert(name.clone(), fields.clone());
                    current = None;
                } else if let Some(field) = line.strip_prefix("    pub ").or_else(|| line.strip_prefix("    ")).and_then(|field| field.split_once(':')) {
                    if !field.0.contains(' ') && !field.0.starts_with('/') && !field.0.starts_with('#') {
                        fields.push(field.0.to_owned());
                    }
                }
            } else if let Some(name) = line.strip_prefix("#[schemars(rename = \"").and_then(|name| name.split('"').next()) {
                renamed = Some(name.to_owned());
            } else if let Some(name) = line.strip_prefix("pub struct ").or_else(|| line.strip_prefix("struct ")).and_then(|name| name.strip_suffix(" {")) {
                current = Some((renamed.take().unwrap_or_else(|| name.to_owned()), vec![]));
            }
        }
    }

    orders
}

/// Returns all the pages, with their methods in the order they're shown.
fn pages() -> Vec<Page> {
    let mut pages = vec![];

    let mut page = Page::new("Session and dependencies", "session", include_str!("session.rs"));
    page.method::<session::GetSessionStatus>();
    page.method::<session::Configure>();
    page.method::<session::Disconnect>();
    page.method::<session::SetGame>();
    page.method::<session::GenerateDependenciesCache>();
    page.method::<session::RebuildDependencies>();
    page.method::<session::ListDependencyTables>();
    page.method_with_opaque_response::<session::GetDependenciesInfo>();
    page.method::<session::GetDependencyTableData>();
    pages.push(page);

    let mut page = Page::new("Packs", "packs", include_str!("packs.rs"));
    page.method::<packs::GetPackInfo>();
    page.method::<packs::NewPack>();
    page.method::<packs::OpenPack>();
    page.method::<packs::OpenVanillaPacks>();
    page.method::<packs::ClosePack>();
    page.method::<packs::CloseAllPacks>();
    page.method::<packs::SavePack>();
    page.method::<packs::UpdatePack>();
    page.method::<packs::GetPackSettings>();
    page.method::<packs::UpdatePackSettings>();
    pages.push(page);

    let mut page = Page::new("Files", "files", include_str!("files.rs"));
    page.method::<files::ListFiles>();
    page.method::<files::CreateFile>();
    page.method::<files::AddFilesFromDisk>();
    page.method::<files::CopyFiles>();
    page.method::<files::DeleteFiles>();
    page.method::<files::RenameFiles>();
    page.method::<files::DuplicateFiles>();
    page.method::<files::ExtractFiles>();
    page.method::<files::ReadFile>();
    page.method::<files::WriteFile>();
    page.method::<files::ListAnimPack>();
    page.method::<files::AddToAnimPack>();
    page.method::<files::ExtractFromAnimPack>();
    page.method::<files::DeleteFromAnimPack>();
    page.method::<files::GetViewData>();
    page.method::<files::GetFilesInfo>();
    page.method::<files::PasteFiles>();
    page.method::<files::GetFilesFromAllSources>();
    page.method::<files::SaveFiles>();
    page.method::<files::OpenInExternalProgram>();
    page.method::<files::SaveExternalFile>();
    pages.push(page);

    let mut page = Page::new("Tables", "tables", include_str!("tables.rs"));
    page.method::<tables::GetTableInfo>();
    page.method::<tables::GetTableRows>();
    page.method::<tables::GetTableDefinition>();
    page.method::<tables::EditTable>();
    page.method::<tables::GetColumnValues>();
    page.method::<tables::GetUnusedNumbers>();
    page.method::<tables::MergeTables>();
    page.method::<tables::UpgradeTable>();
    page.method::<tables::RenameKey>();
    page.method::<tables::AddKeyDeletes>();
    page.method::<tables::ExportTsv>();
    page.method::<tables::ImportTsv>();
    pages.push(page);

    let mut page = Page::new("Schema", "schema", include_str!("schema.rs"));
    page.method::<schema::ListSchemaTables>();
    page.method::<schema::PatchColumn>();
    page.method::<schema::RemovePatches>();
    page.method::<schema::UpdateSchemas>();
    page.method::<schema::UpdateSchemaFromAssemblyKit>();
    page.method::<schema::GetRawDefinitions>();
    page.method::<schema::SetDefinition>();
    page.method::<schema::DeleteDefinition>();
    page.method::<schema::GetReferencingColumns>();
    page.method::<schema::GetTablePatches>();
    page.method::<schema::ImportPatches>();
    page.method::<schema::GetMissingDefinitions>();
    pages.push(page);

    let mut page = Page::new("References", "references", include_str!("references.rs"));
    page.method::<references::FindDefinition>();
    page.method::<references::FindUsages>();
    page.method::<references::FindLoc>();
    page.method::<references::GetLocSource>();
    page.method::<references::GetRowLocs>();
    page.method::<references::GetTableReferenceData>();
    page.method::<references::GetReferenceValues>();
    pages.push(page);

    let mut page = Page::new("Diagnostics", "diagnostics", include_str!("diagnostics.rs"));
    page.method::<diagnostics::RunDiagnostics>();
    page.method::<diagnostics::ListDiagnostics>();
    page.method_with_opaque_response::<diagnostics::GetDiagnosticsReport>();
    page.method::<diagnostics::IgnoreDiagnostics>();
    pages.push(page);

    let mut page = Page::new("Search", "search", include_str!("search.rs"));
    page.method::<search::GetSearchReport>();
    page.method::<search::RunSearch>();
    page.method::<search::ListSearchMatches>();
    page.method::<search::ReplaceSearchMatches>();
    pages.push(page);

    let mut page = Page::new("Notes", "notes", include_str!("notes.rs"));
    page.method::<notes::ListNotes>();
    page.method::<notes::AddNote>();
    page.method::<notes::DeleteNote>();
    pages.push(page);

    let mut page = Page::new("Tools", "tools", include_str!("tools.rs"));
    page.method::<tools::GetOptimizerOptions>();
    page.method::<tools::OptimizePack>();
    page.method::<tools::PatchSiegeAi>();
    page.method::<tools::PackMap>();
    page.method::<tools::GenerateMissingLocs>();
    page.method::<tools::UpdateAnimIds>();
    page.method::<tools::AnimsBySkeleton>();
    page.method::<tools::ExportGltf>();
    page.method::<tools::SetVideoFormat>();
    page.method::<tools::LiveExport>();
    page.method::<tools::InitMyMod>();
    page.method::<tools::RunLuaTests>();
    page.method::<tools::GetStartposCampaigns>();
    page.method::<tools::StartStartpos>();
    page.method::<tools::FinishStartpos>();
    page.method::<tools::ValidateStartpos>();
    page.method::<tools::GenerateStartpos>();
    page.method::<tools::ListPluginScripts>();
    page.method::<tools::RunPluginScript>();
    page.method::<tools::GetLuaHovers>();
    page.method::<tools::ListTraitCeos>();
    page.method::<tools::AddCeoEntries>();
    page.method::<tools::BuildCeo>();
    page.method::<tools::ImportCeo>();
    pages.push(page);

    let mut page = Page::new("Translations", "translations", include_str!("translations.rs"));
    page.method_with_opaque_response::<translations::GetPackTranslation>();
    page.method::<translations::SubmitTranslation>();
    page.method::<translations::ListTranslations>();
    page.method::<translations::GenerateVanillaTexts>();
    pages.push(page);

    let mut page = Page::new("GitHub", "github", include_str!("github.rs"));
    page.method_with_opaque_response::<github::StartGitHubSignIn>();
    page.method::<github::PollGitHubSignIn>();
    page.method::<github::GetGitHubAccount>();
    page.method::<github::SignOutOfGitHub>();
    pages.push(page);

    let mut page = Page::new("Updates", "updates", include_str!("updates.rs"));
    page.method::<updates::CheckUpdate>();
    page.method::<updates::ApplyUpdate>();
    pages.push(page);

    let mut page = Page::new("Jobs", "jobs", include_str!("jobs.rs"));
    page.method::<jobs::GetJobStatus>();
    page.method::<jobs::WaitForJob>();
    page.method::<jobs::CancelJob>();
    pages.push(page);

    pages
}

/// Returns the index page, with every method and a link to its page.
fn index(pages: &[Page]) -> String {
    let mut text = format!("{GENERATED_NOTICE}\n\n# Methods\n\n{INDEX_INTRO}\n");
    for page in pages {
        let definitions = page.definitions();
        text.push_str(&format!("\n## [{}](./{})\n\n| Method | Description |\n|--------|-------------|\n", page.title, page.path()));
        for method in &page.methods {
            let description = method_description(method.name, resolve(&method.request, &definitions));
            let first_paragraph = description.split("\n\n").next().unwrap_or_default();
            let job = if method.is_job { " Runs as a job." } else { "" };
            let anchor = method.name.replace('.', "");
            text.push_str(&format!("| [`{}`](./{}#{anchor}) | {}{job} |\n", method.name, page.path(), table_cell(first_paragraph)));
        }
    }

    text
}

#[test]
fn method_docs() {
    let pages = pages();

    // Every method in the API must have a page.
    let documented = pages.iter().flat_map(|page| page.methods.iter().map(|method| method.request_type)).collect::<BTreeSet<_>>();
    let defined = pages.iter()
        .flat_map(|page| page.source.lines())
        .filter_map(|line| line.trim().strip_prefix("impl Request for "))
        .filter_map(|line| line.split_whitespace().next())
        .collect::<BTreeSet<_>>();
    assert_eq!(documented, defined, "Every method must be added to the pages in `method_docs.rs`.");

    // Every page must be in the book, or mdbook won't build it.
    let summary = fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../docs/SUMMARY.md")).expect("Failed to read the book's summary.");
    let missing = pages.iter().map(Page::path).filter(|path| !summary.contains(&format!("(./server/{path})"))).collect::<Vec<_>>();
    assert!(missing.is_empty(), "These doc pages are missing from docs/SUMMARY.md: {missing:?}.");

    let orders = field_orders(&pages);
    let mut files = vec![("methods.md".to_owned(), index(&pages))];
    files.extend(pages.iter().map(|page| (page.path(), page.render(&orders))));

    let docs_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../docs/server");
    let update = std::env::var_os(UPDATE_DOCS_VAR).is_some();
    let mut outdated = vec![];
    for (path, text) in files {
        let full_path = docs_path.join(&path);
        if update {
            fs::create_dir_all(full_path.parent().expect("Doc pages are inside docs/server.")).expect("Failed to create the methods folder.");
            fs::write(&full_path, text).expect("Failed to write a doc page.");
        } else if fs::read_to_string(&full_path).ok().as_deref() != Some(text.as_str()) {
            outdated.push(format!("docs/server/{path}"));
        }
    }

    assert!(outdated.is_empty(), "These doc pages are outdated: {outdated:?}. Run `{UPDATE_DOCS_VAR}=1 cargo test -p rpfm_ipc method_docs` to update them.");
}
