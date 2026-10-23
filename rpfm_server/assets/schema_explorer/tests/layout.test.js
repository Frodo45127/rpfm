// Run with: node --test "rpfm_server/assets/schema_explorer/tests/*.test.js"

const test = require("node:test");
const assert = require("node:assert/strict");

// The graphology-library bundle touches these browser globals when it loads (for its GEXF/GraphML parsers).
globalThis.DOMParser ??= class {};
globalThis.XMLSerializer ??= class {};
globalThis.Document ??= class {};

const { computeLayout, splitGraph, cacheKey, separateStars, starRadius } = require("../js/layout.js");

function edge(from, to) {
    return { from, from_column: "key", to, to_column: "key", lookup: false };
}

// Two triangles joined by c -> d, a self-reference on a, and two isolated tables.
function sampleGraph() {
    const ids = ["a", "b", "c", "d", "e", "f", "lonely_1", "lonely_2"];
    return {
        nodes: ids.map((id) => ({ id })),
        edges: [edge("a", "b"), edge("b", "c"), edge("c", "a"), edge("b", "a"), edge("c", "d"), edge("d", "e"), edge("e", "f"), edge("f", "d"), edge("a", "a")],
    };
}

test("splitGraph separates isolated tables and keeps one link per pair", () => {
    const split = splitGraph(sampleGraph());

    assert.deepEqual(split.isolated, ["lonely_1", "lonely_2"]);
    assert.deepEqual(split.connected, ["a", "b", "c", "d", "e", "f"]);
    assert.equal(split.links.length, 7);
});

test("a table that only references itself counts as isolated", () => {
    const split = splitGraph({ nodes: [{ id: "self" }], edges: [edge("self", "self")] });

    assert.deepEqual(split.isolated, ["self"]);
    assert.deepEqual(split.links, []);
});

test("computeLayout positions every table and is deterministic", async () => {
    const first = await computeLayout(sampleGraph());
    const second = await computeLayout(sampleGraph());

    assert.deepEqual(Object.keys(first.positions).sort(), sampleGraph().nodes.map((node) => node.id).sort());
    for (const { x, y } of Object.values(first.positions)) {
        assert.ok(Number.isFinite(x) && Number.isFinite(y));
    }
    assert.deepEqual(first.positions, second.positions);
});

test("isolated tables are clustered to the right of the graph", async () => {
    const { positions, isolated } = await computeLayout(sampleGraph());

    const rightmostConnected = Math.max(...["a", "b", "c", "d", "e", "f"].map((id) => positions[id].x));
    assert.deepEqual(isolated, ["lonely_1", "lonely_2"]);
    for (const id of isolated) {
        assert.ok(positions[id].x > rightmostConnected);
    }
});

test("progress is reported up to 1", async () => {
    const seen = [];

    await computeLayout(sampleGraph(), (progress) => seen.push(progress));

    assert.ok(seen.length > 1);
    assert.equal(seen[seen.length - 1], 1);
    assert.ok(seen.every((value, index) => index === 0 || value >= seen[index - 1]));
});

test("separateStars pushes overlapping stars apart, including ones on the same spot", () => {
    const stars = [
        { x: 0, y: 0, radius: 5 },
        { x: 2, y: 0, radius: 5 },
        { x: 2, y: 0, radius: 5 },
        { x: 100, y: 100, radius: 5 },
    ];

    const left = separateStars(stars);

    assert.equal(left, 0);
    for (let i = 0; i < stars.length; i++) {
        for (let j = i + 1; j < stars.length; j++) {
            const distance = Math.hypot(stars[i].x - stars[j].x, stars[i].y - stars[j].y);
            assert.ok(distance >= stars[i].radius + stars[j].radius, `stars ${i} and ${j} still overlap`);
        }
    }
    assert.deepEqual(stars[3], { x: 100, y: 100, radius: 5 });
});

test("starRadius grows with the number of linked tables and caps", () => {
    assert.ok(starRadius(0) < starRadius(10));
    assert.equal(starRadius(120), starRadius(500));
});

test("the cache key changes when the graph changes", () => {
    const graph = sampleGraph();
    const changed = { ...graph, edges: [...graph.edges, edge("lonely_1", "a")] };

    assert.equal(cacheKey(graph), cacheKey(sampleGraph()));
    assert.notEqual(cacheKey(graph), cacheKey(changed));
});
