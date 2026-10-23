// Run with: node --test "rpfm_server/assets/schema_explorer/tests/*.test.js"

const test = require("node:test");
const assert = require("node:assert/strict");

const { GraphIndex, pairKey } = require("../js/graph_index.js");

// a -> hub, b -> hub, c -> hub, d -> hub, a -> b, b -> c, e is isolated.
function sampleGraph() {
    const ids = ["a", "b", "c", "d", "e", "hub"];
    const edge = (from, to, column = "key") => ({ from, from_column: column, to, to_column: "key", lookup: false });
    return {
        nodes: ids.map((id) => ({ id, table: `${id}_tables`, missing: false })),
        edges: [edge("a", "hub"), edge("b", "hub"), edge("c", "hub"), edge("d", "hub"), edge("a", "b"), edge("a", "b", "other"), edge("b", "c")],
    };
}

test("degree counts distinct neighbouring tables", () => {
    const index = new GraphIndex(sampleGraph());

    assert.equal(index.degree("hub"), 4);
    assert.equal(index.degree("a"), 2);
    assert.equal(index.degree("e"), 0);
});

test("neighbours follow the requested direction", () => {
    const index = new GraphIndex(sampleGraph());

    assert.deepEqual(index.neighbours("b", "out").sort(), ["c", "hub"]);
    assert.deepEqual(index.neighbours("b", "in"), ["a"]);
    assert.deepEqual(index.neighbours("b", "both").sort(), ["a", "c", "hub"]);
    assert.throws(() => index.neighbours("b", "sideways"));
});

test("shortestPath follows references either way", () => {
    const index = new GraphIndex(sampleGraph());

    assert.deepEqual(index.shortestPath("a", "d"), ["a", "hub", "d"]);
    assert.deepEqual(index.shortestPath("d", "b"), ["d", "hub", "b"]);
    assert.equal(index.shortestPath("a", "e"), null);
    assert.equal(index.shortestPath("a", "nowhere"), null);
    assert.deepEqual(index.shortestPath("a", "a"), ["a"]);
});

test("edgesBetween returns every column in both directions", () => {
    const index = new GraphIndex(sampleGraph());

    const columns = index.edgesBetween("b", "a").map((edge) => `${edge.from}.${edge.from_column}`);

    assert.deepEqual(columns, ["a.key", "a.other"]);
});

test("linkedPairs gives one entry per table pair with the references each way", () => {
    const index = new GraphIndex({
        nodes: ["a", "b", "c"].map((id) => ({ id })),
        edges: [
            { from: "a", from_column: "x", to: "b", to_column: "key" },
            { from: "b", from_column: "y", to: "a", to_column: "key" },
            { from: "c", from_column: "z", to: "a", to_column: "key" },
            { from: "c", from_column: "parent", to: "c", to_column: "key" },
        ],
    });

    const pairs = new Map(index.linkedPairs().map((pair) => [pair.key, pair]));

    assert.deepEqual([...pairs.keys()].sort(), ["a|b", "a|c", "c|c"]);
    assert.deepEqual(pairs.get("a|b").forward.map((edge) => edge.from_column), ["x"]);
    assert.deepEqual(pairs.get("a|b").backward.map((edge) => edge.from_column), ["y"]);
    assert.deepEqual(pairs.get("a|c").forward, []);
    assert.deepEqual(pairs.get("a|c").backward.map((edge) => edge.from_column), ["z"]);
    assert.equal(pairs.get("c|c").a, "c");
});

test("pairKey is the same whichever way round the tables are given", () => {
    assert.equal(pairKey("units", "factions"), pairKey("factions", "units"));
    assert.equal(pairKey("factions", "units"), "factions|units");
});

test("search ranks exact, prefix, word prefix, substring, then subsequence matches", () => {
    const index = new GraphIndex({
        nodes: ["battles", "battles_extra", "custom_battles", "notbattles", "b_a_t_t_l_e_s_x", "units"].map((id) => ({ id })),
        edges: [],
    });

    const results = index.search("battles", 10).map((node) => node.id);

    assert.deepEqual(results, ["battles", "battles_extra", "custom_battles", "notbattles", "b_a_t_t_l_e_s_x"]);
    assert.deepEqual(index.search("battles_tables", 1).map((node) => node.id), ["battles"]);
    assert.deepEqual(index.search("   ", 10), []);
});
