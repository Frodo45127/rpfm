// Run with: node --test "rpfm_server/assets/schema_explorer/tests/*.test.js"

const test = require("node:test");
const assert = require("node:assert/strict");

const { GraphIndex } = require("../js/graph_index.js");

// a -> hub, b -> hub, c -> hub, d -> hub, a -> b, b -> c, e is isolated.
function sampleGraph() {
    const ids = ["a", "b", "c", "d", "e", "hub"];
    const edge = (from, to, column = "key") => ({ from, from_column: column, to, to_column: "key", lookup: false });
    return {
        nodes: ids.map((id) => ({ id, table: `${id}_tables`, missing: false })),
        edges: [edge("a", "hub"), edge("b", "hub"), edge("c", "hub"), edge("d", "hub"), edge("a", "b"), edge("a", "b", "other"), edge("b", "c")],
    };
}

const DEFAULTS = { depth: 1, direction: "both", hubThreshold: 3, hideHubs: false, maxNodes: 100 };

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

test("neighbourhood doesn't expand through hubs", () => {
    const index = new GraphIndex(sampleGraph());

    const { ids } = index.neighbourhood("a", { ...DEFAULTS, depth: 2 });

    assert.deepEqual([...ids].sort(), ["a", "b", "c", "hub"]);
});

test("a focused hub is expanded", () => {
    const index = new GraphIndex(sampleGraph());

    const { ids } = index.neighbourhood("hub", DEFAULTS);

    assert.deepEqual([...ids].sort(), ["a", "b", "c", "d", "hub"]);
});

test("hideHubs leaves hubs out of the neighbourhood", () => {
    const index = new GraphIndex(sampleGraph());

    const { ids } = index.neighbourhood("a", { ...DEFAULTS, hideHubs: true });

    assert.deepEqual([...ids].sort(), ["a", "b"]);
});

test("maxNodes truncates the neighbourhood", () => {
    const index = new GraphIndex(sampleGraph());

    const result = index.neighbourhood("hub", { ...DEFAULTS, maxNodes: 3 });

    assert.equal(result.ids.size, 3);
    assert.equal(result.truncated, true);
});

test("shortestPath respects direction and can avoid hubs", () => {
    const index = new GraphIndex(sampleGraph());

    assert.deepEqual(index.shortestPath("a", "d", { direction: "both", avoidHubsOver: null }), ["a", "hub", "d"]);
    assert.equal(index.shortestPath("a", "d", { direction: "both", avoidHubsOver: 3 }), null);
    assert.deepEqual(index.shortestPath("a", "c", { direction: "out", avoidHubsOver: 3 }), ["a", "b", "c"]);
    assert.equal(index.shortestPath("c", "a", { direction: "out", avoidHubsOver: null }), null);
    assert.equal(index.shortestPath("a", "e", { direction: "both", avoidHubsOver: null }), null);
    assert.deepEqual(index.shortestPath("a", "a", { direction: "both", avoidHubsOver: null }), ["a"]);
});

test("edgesBetween returns every column in both directions", () => {
    const index = new GraphIndex(sampleGraph());

    const columns = index.edgesBetween("b", "a").map((edge) => `${edge.from}.${edge.from_column}`);

    assert.deepEqual(columns, ["a.key", "a.other"]);
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
