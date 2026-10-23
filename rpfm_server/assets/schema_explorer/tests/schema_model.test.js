// Run with: node --test "rpfm_server/assets/schema_explorer/tests/*.test.js"

const test = require("node:test");
const assert = require("node:assert/strict");

const { SchemaModel, parseSchemaText, normalizeSchema, patchedField, tableId } = require("../js/schema_model.js");

function field(name, extra = {}) {
    return {
        name,
        field_type: "StringU8",
        is_key: false,
        default_value: null,
        is_filename: false,
        filename_relative_path: null,
        is_reference: null,
        lookup: null,
        description: "",
        ca_order: -1,
        is_bitwise: 0,
        enum_values: {},
        is_part_of_colour: null,
        ...extra,
    };
}

function sampleSchema() {
    return {
        version: 5,
        definitions: {
            battles_tables: [
                { version: 16, fields: [field("key", { is_key: true }), field("type", { is_reference: ["battle_types", "type"] })], localised_fields: [] },
                {
                    version: 17,
                    fields: [
                        field("key", { is_key: true }),
                        field("type", { is_reference: ["battle_types", "type"], lookup: ["onscreen"] }),
                        field("culture", { is_reference: ["cultures", "key"] }),
                    ],
                    localised_fields: [field("localised_name")],
                },
            ],
            battle_types_tables: [{ version: 1, fields: [field("type", { is_key: true })], localised_fields: [] }],
            battle_sources_tables: [
                { version: 1, fields: [field("battle", { is_reference: ["battles", "key"] })], localised_fields: [] },
                { version: 2, fields: [field("battle_key", { is_reference: ["battles", "key"] })], localised_fields: [] },
            ],
        },
        patches: {
            battle_types_tables: { type: { description: "Patched description", explanation: "Shown as a note" } },
        },
    };
}

test("tableId strips the _tables suffix only", () => {
    assert.equal(tableId("battles_tables"), "battles");
    assert.equal(tableId("battles"), "battles");
});

test("graph has one node per table and edges from the newest definition only", () => {
    const model = new SchemaModel(sampleSchema());

    const graph = model.graph();

    const battles = graph.nodes.find((node) => node.id === "battles");
    assert.deepEqual(battles, {
        id: "battles",
        table: "battles_tables",
        versions: [17, 16],
        fields: 3,
        keys: ["key"],
        localised: true,
        missing: false,
    });
    const battleEdges = graph.edges.filter((edge) => edge.from === "battles");
    assert.deepEqual(battleEdges, [
        { from: "battles", from_column: "type", to: "battle_types", to_column: "type", lookup: true },
        { from: "battles", from_column: "culture", to: "cultures", to_column: "key", lookup: false },
    ]);
    assert.deepEqual(graph.edges.filter((edge) => edge.from === "battle_sources").map((edge) => edge.from_column), ["battle_key"]);
});

test("referenced tables missing from the schema get a missing node", () => {
    const model = new SchemaModel(sampleSchema());

    const cultures = model.graph().nodes.find((node) => node.id === "cultures");

    assert.equal(cultures.missing, true);
    assert.equal(cultures.table, "cultures_tables");
    assert.deepEqual(cultures.versions, []);
});

test("referenced_by covers every version of the referencing tables, like rpfm_lib", () => {
    const model = new SchemaModel(sampleSchema());

    const details = model.tableDetails("battles_tables");

    assert.deepEqual(details.referenced_by, { key: { battle_sources_tables: ["battle", "battle_key"] } });
    assert.deepEqual(details.definitions.map((definition) => definition.version), [17, 16]);
});

test("details of a missing table only list who references it", () => {
    const model = new SchemaModel(sampleSchema());

    const details = model.tableDetails("cultures");

    assert.equal(details.missing, true);
    assert.deepEqual(details.definitions, []);
    assert.deepEqual(details.referenced_by, { key: { battles_tables: ["culture"] } });
    assert.equal(model.tableDetails("nothing_here"), null);
});

test("patches replace field values and other patch keys become notes", () => {
    const model = new SchemaModel(sampleSchema());

    const type = model.tableDetails("battle_types").definitions[0].fields[0];

    assert.equal(type.description, "Patched description");
    assert.deepEqual(type.patch_notes, { explanation: "Shown as a note" });
});

test("patchedField parses patch values the way rpfm_lib's Field accessors do", () => {
    const patches = {
        a: { is_key: "true", is_reference: "units;key", lookup: "name;icon", is_filename: "true", filename_relative_path: "ui\\icons\\%.png;ui/other" },
        b: { is_reference: "no_separator" },
    };

    const a = patchedField(field("a"), patches);
    const b = patchedField(field("b", { is_reference: ["kept", "column"] }), patches);

    assert.equal(a.is_key, true);
    assert.deepEqual(a.is_reference, ["units", "key"]);
    assert.deepEqual(a.lookup, ["name", "icon"]);
    assert.equal(a.is_filename, true);
    assert.deepEqual(a.filename_relative_path, ["ui/icons/%.png", "ui/other"]);
    assert.deepEqual(b.is_reference, ["kept", "column"]);
});

test("schemas load from RON, plain JSON, a Schema wrapper and a saved MCP reply", () => {
    const schema = sampleSchema();
    const json = JSON.stringify(schema);
    const mcpReply = JSON.stringify({ jsonrpc: "2.0", id: 4, result: { content: [{ type: "text", text: JSON.stringify({ Schema: schema }) }] } });
    const ron = `(version: 5, definitions: {"x_tables": [(version: 1, fields: [], localised_fields: [], localised_key_order: [])]}, patches: {})`;

    assert.deepEqual(parseSchemaText(json).definitions, schema.definitions);
    assert.deepEqual(parseSchemaText(JSON.stringify({ Schema: schema })).definitions, schema.definitions);
    assert.deepEqual(parseSchemaText(mcpReply).definitions, schema.definitions);
    assert.deepEqual(Object.keys(parseSchemaText(ron).definitions), ["x_tables"]);
    assert.throws(() => normalizeSchema({ hello: "world" }), /doesn't look like an RPFM schema/);
});
