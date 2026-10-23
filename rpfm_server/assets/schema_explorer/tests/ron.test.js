// Run with: node --test "rpfm_server/assets/schema_explorer/tests/*.test.js"

const test = require("node:test");
const assert = require("node:assert/strict");

const { parse, RonError } = require("../js/ron.js");

test("structs, lists and maps parse like serde_json shapes", () => {
    const text = `(
        version: 5,
        definitions: {
            "a_tables": [
                (version: 2, fields: [], localised_fields: [],),
            ],
        },
        patches: {},
    )`;

    const value = parse(text);

    assert.deepEqual(value, {
        version: 5,
        definitions: { a_tables: [{ version: 2, fields: [], localised_fields: [] }] },
        patches: {},
    });
});

test("Some, None, booleans and tuples", () => {
    const value = parse(`(a: Some(("t", "c")), b: None, c: true, d: false, e: Some("x"))`);

    assert.deepEqual(value, { a: ["t", "c"], b: null, c: true, d: false, e: "x" });
});

test("enum variants become strings or single-key objects", () => {
    const value = parse(`[StringU8, SequenceU32((version: 1)), Pair(1, 2)]`);

    assert.deepEqual(value, ["StringU8", { SequenceU32: { version: 1 } }, { Pair: [1, 2] }]);
});

test("numbers, including negatives, floats, exponents, hex and underscores", () => {
    const value = parse(`[-1, 0, 2.5, -0.25, 1e3, 0xff, 1_000, inf, NaN]`);

    assert.deepEqual(value.slice(0, 7), [-1, 0, 2.5, -0.25, 1000, 255, 1000]);
    assert.equal(value[7], Infinity);
    assert.ok(Number.isNaN(value[8]));
});

test("map keys are strings, including integer keys", () => {
    const value = parse(`{0: "zero", 12: "twelve"}`);

    assert.deepEqual(value, { 0: "zero", 12: "twelve" });
});

test("string escapes and raw strings", () => {
    const value = parse(String.raw`["a\"b", "line\nbreak", "tab\t", "back\\slash", "\u{48}i", r#"raw "quoted""#]`);

    assert.deepEqual(value, ["a\"b", "line\nbreak", "tab\t", "back\\slash", "Hi", "raw \"quoted\""]);
});

test("multibyte text is kept as is", () => {
    const value = parse(`(description: "Kislev Ледяная ведьма, 冰雪女巫")`);

    assert.equal(value.description, "Kislev Ледяная ведьма, 冰雪女巫");
});

test("comments are skipped", () => {
    const value = parse(`// header
        (a: 1, /* inline */ b: 2)`);

    assert.deepEqual(value, { a: 1, b: 2 });
});

test("identifiers starting with an underscore or 'inf' aren't numbers", () => {
    const value = parse(`[_Private, Infantry, info]`);

    assert.deepEqual(value, ["_Private", "Infantry", "info"]);
});

test("invalid input reports where it failed", () => {
    assert.throws(() => parse(`(a: 1,\n b: )`), (error) => error instanceof RonError && /line 2/.test(error.message));
    assert.throws(() => parse(`"unterminated`), RonError);
    assert.throws(() => parse(`(a: 1) trailing`), RonError);
});
