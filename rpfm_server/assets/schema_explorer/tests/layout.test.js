// Run with: node --test "rpfm_server/assets/schema_explorer/tests/*.test.js"

const test = require("node:test");
const assert = require("node:assert/strict");

// The graphology-library bundle touches these browser globals when it loads (for its GEXF/GraphML parsers).
globalThis.DOMParser ??= class {};
globalThis.XMLSerializer ??= class {};
globalThis.Document ??= class {};

const { computeLayout, layoutFor, saveMovedStars, clearLegacyCaches, splitGraph, graphSignature, separateStars, starRadius, spiralSlot } = require("../js/layout.js");

// A Map-backed stand-in for window.localStorage.
function installStorage() {
    const items = new Map();
    globalThis.window = {
        localStorage: {
            get length() {
                return items.size;
            },
            key: (index) => [...items.keys()][index] ?? null,
            getItem: (key) => (items.has(key) ? items.get(key) : null),
            setItem: (key, value) => items.set(key, String(value)),
            removeItem: (key) => items.delete(key),
        },
    };
    return items;
}

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

function withNewTable(graph) {
    return {
        nodes: [...graph.nodes, { id: "g" }, { id: "lonely_3" }],
        edges: [...graph.edges, edge("g", "e"), edge("g", "f")],
    };
}

function distance(p, q) {
    return Math.hypot(p.x - q.x, p.y - q.y);
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

test("computeLayout positions every table, converges, and is deterministic", async () => {
    const first = await computeLayout(sampleGraph());
    const second = await computeLayout(sampleGraph());

    assert.deepEqual(Object.keys(first.positions).sort(), sampleGraph().nodes.map((node) => node.id).sort());
    for (const { x, y } of Object.values(first.positions)) {
        assert.ok(Number.isFinite(x) && Number.isFinite(y));
    }
    assert.ok(first.iterations > 0 && first.iterations <= 5000);
    assert.ok(first.scale > 0);
    assert.equal(first.signature, graphSignature(sampleGraph()));
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

test("progress reports growing iteration counts and a movement value", async () => {
    const seen = [];

    await computeLayout(sampleGraph(), { onProgress: (progress) => seen.push(progress) });

    assert.ok(seen.length > 0);
    assert.ok(seen.every((progress, index) => Number.isFinite(progress.movement) && (index === 0 || progress.iterations > seen[index - 1].iterations)));
});

test("an updated schema continues from the previous layout, keeping pinned stars and the cluster in place", async () => {
    const previous = await computeLayout(sampleGraph());
    previous.positions.a = { x: previous.positions.a.x + 123.4, y: previous.positions.a.y - 56.7 };
    previous.pinned = ["a"];

    const updated = await computeLayout(withNewTable(sampleGraph()), { previous });

    assert.deepEqual(updated.positions.a, previous.positions.a);
    assert.deepEqual(updated.pinned, ["a"]);
    assert.equal(updated.scale, previous.scale);
    assert.deepEqual(updated.positions.lonely_1, previous.positions.lonely_1);
    assert.deepEqual(updated.positions.lonely_2, previous.positions.lonely_2);
    assert.deepEqual(updated.clusterCentre, previous.clusterCentre);
    const fromCentre = (id, layout) => distance(layout.positions[id], layout.clusterCentre);
    assert.ok(fromCentre("lonely_3", updated) > Math.max(fromCentre("lonely_1", previous), fromCentre("lonely_2", previous)), "new isolated tables continue the spiral outward");

    // The new table starts between its neighbours e and f, so it ends up nearer them than the far triangle.
    const p = updated.positions;
    assert.ok(distance(p.g, p.e) < distance(p.g, p.b));
});

test("tables without references sit on a spiral around the cluster centre, without overlapping", async () => {
    const graph = sampleGraph();
    for (let i = 3; i <= 40; i++) {
        graph.nodes.push({ id: `lonely_${i}` });
    }

    const { positions, isolated, clusterCentre } = await computeLayout(graph);

    assert.equal(isolated.length, 40);
    const spiral = isolated.map((id) => positions[id]);
    for (let i = 0; i < spiral.length; i++) {
        for (let j = i + 1; j < spiral.length; j++) {
            assert.ok(distance(spiral[i], spiral[j]) > 2 * starRadius(0), `isolated tables ${i} and ${j} overlap`);
        }
    }
    const radii = isolated.map((id) => distance(positions[id], clusterCentre));
    assert.ok(radii.every((radius, index) => index === 0 || radius > radii[index - 1]), "each slot is farther out than the previous one");
});

test("spiralSlot spirals outward from the centre", () => {
    const centre = { x: 100, y: -50 };

    const inner = spiralSlot(centre, 0);
    const outer = spiralSlot(centre, 50);

    assert.ok(distance(inner, centre) < distance(outer, centre));
    assert.deepEqual(spiralSlot(centre, 7), spiralSlot(centre, 7));
});

test("separateStars pushes overlapping stars apart, including ones on the same spot, and never moves fixed ones", () => {
    const stars = [
        { x: 0, y: 0, radius: 5, fixed: true },
        { x: 2, y: 0, radius: 5 },
        { x: 2, y: 0, radius: 5 },
        { x: 100, y: 100, radius: 5 },
    ];

    const left = separateStars(stars);

    assert.equal(left, 0);
    for (let i = 0; i < stars.length; i++) {
        for (let j = i + 1; j < stars.length; j++) {
            assert.ok(distance(stars[i], stars[j]) >= stars[i].radius + stars[j].radius, `stars ${i} and ${j} still overlap`);
        }
    }
    assert.deepEqual(stars[0], { x: 0, y: 0, radius: 5, fixed: true });
    assert.deepEqual(stars[3], { x: 100, y: 100, radius: 5 });
});

test("starRadius grows with the number of linked tables and caps", () => {
    assert.ok(starRadius(0) < starRadius(10));
    assert.equal(starRadius(120), starRadius(500));
});

test("the graph signature changes when the graph changes", () => {
    assert.equal(graphSignature(sampleGraph()), graphSignature(sampleGraph()));
    assert.notEqual(graphSignature(sampleGraph()), graphSignature(withNewTable(sampleGraph())));
});

test("layoutFor computes, then reuses, then updates the saved layout", async () => {
    installStorage();

    const fresh = await layoutFor("file:test.ron", sampleGraph());
    const again = await layoutFor("file:test.ron", sampleGraph());
    const updated = await layoutFor("file:test.ron", withNewTable(sampleGraph()));

    assert.equal(fresh.status, "new");
    assert.equal(again.status, "cached");
    assert.deepEqual(again.positions, fresh.positions);
    assert.equal(updated.status, "updated");
    assert.ok(updated.positions.g);
});

test("saveMovedStars stores and pins moved stars in the saved layout", async () => {
    installStorage();
    await layoutFor("file:test.ron", sampleGraph());

    const saved = saveMovedStars("file:test.ron", { b: { x: 1000.04, y: -500 }, unknown: { x: 0, y: 0 } });
    const reloaded = await layoutFor("file:test.ron", sampleGraph());

    assert.equal(saved, true);
    assert.equal(reloaded.status, "cached");
    assert.deepEqual(reloaded.positions.b, { x: 1000, y: -500 });
    assert.deepEqual(reloaded.pinned, ["b"]);
    assert.equal(reloaded.positions.unknown, undefined);
    assert.equal(saveMovedStars("file:never-loaded.ron", { b: { x: 0, y: 0 } }), false);
});

test("moving the whole cluster saves its stars unpinned and the new spiral centre", async () => {
    installStorage();
    const layout = await layoutFor("file:test.ron", sampleGraph());
    const moved = { x: layout.clusterCentre.x + 100, y: layout.clusterCentre.y };

    saveMovedStars("file:test.ron", { lonely_1: { x: 1, y: 2 } }, { pin: false, clusterCentre: moved });
    const reloaded = await layoutFor("file:test.ron", sampleGraph());

    assert.deepEqual(reloaded.positions.lonely_1, { x: 1, y: 2 });
    assert.deepEqual(reloaded.pinned, []);
    assert.deepEqual(reloaded.clusterCentre, { x: Math.round(moved.x * 10) / 10, y: Math.round(moved.y * 10) / 10 });
});

test("a saved layout from before the spiral is updated, moving unpinned isolated tables onto a spiral", async () => {
    const items = installStorage();
    const layout = await layoutFor("file:test.ron", sampleGraph());
    const old = { ...layout, positions: { ...layout.positions, lonely_1: { x: 5000, y: 5000 } } };
    delete old.clusterCentre;
    delete old.status;
    items.set("rpfm-schema-explorer-layout-v3:file:test.ron", JSON.stringify(old));

    const updated = await layoutFor("file:test.ron", sampleGraph());

    assert.equal(updated.status, "updated");
    assert.ok(updated.clusterCentre);
    assert.ok(distance(updated.positions.lonely_1, updated.clusterCentre) < 200);
});

test("clearLegacyCaches removes layouts saved by older versions only", () => {
    const items = installStorage();
    items.set("rpfm-schema-explorer-layout:abc123", "{}");
    items.set("rpfm-schema-explorer-layout-v3:file:test.ron", "{}");
    items.set("rpfm-schema-explorer-settings", "{}");

    clearLegacyCaches();

    assert.deepEqual([...items.keys()].sort(), ["rpfm-schema-explorer-layout-v3:file:test.ron", "rpfm-schema-explorer-settings"]);
});
