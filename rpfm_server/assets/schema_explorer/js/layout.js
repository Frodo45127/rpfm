// ForceAtlas2 layout of the whole schema graph, run until it converges and kept in localStorage.
//
// One layout is kept per schema source (a game, or a schema file name). When the schema changes, the
// saved layout is the starting point: known tables keep their place, new ones start next to their
// neighbours, and ForceAtlas2 runs again from there. Stars the user dragged are pinned and never moved
// by the layout. Tables without any reference are left out of the force layout and placed on a
// sunflower spiral beside the graph.

(function (root, factory) {
    const graphology = root.graphology || (typeof require === "function" ? require("../vendor/graphology.umd.min.js") : null);
    const library = root.graphologyLibrary || (typeof require === "function" ? require("../vendor/graphology-library.min.js") : null);
    const api = factory(graphology, library);
    if (typeof module === "object" && module.exports) {
        module.exports = api;
    } else {
        root.RpfmLayout = api;
    }
})(typeof self !== "undefined" ? self : this, (graphology, library) => {
    "use strict";

    const CACHE_PREFIX = "rpfm-schema-explorer-layout-v3:";
    const LEGACY_CACHE_PREFIXES = ["rpfm-schema-explorer-layout:"];

    // Convergence: stop once stars move less than this fraction of a typical edge per iteration,
    // or once movement stops improving (Barnes-Hut leaves a small jitter that never goes away).
    const ITERATIONS_PER_CHUNK = 50;
    const MAX_ITERATIONS = 5000;
    const CONVERGED_MOVEMENT = 0.0015;
    const PLATEAU_CHUNKS = 8;
    const PLATEAU_MIN_GAIN = 0.05;

    const TARGET_EDGE_LENGTH = 70;
    const MAX_SEPARATION_PASSES = 50;
    const STAR_MARGIN = 3;
    const DISTANCE_TOLERANCE = 1e-6;
    const SPIRAL_STEP = 24;
    const GOLDEN_ANGLE = Math.PI * (3 - Math.sqrt(5));
    const CLUSTER_GAP = 250;

    // FNV-1a, enough to tell two versions of a schema graph apart.
    function hashString(text) {
        let hash = 0x811c9dc5;
        for (let i = 0; i < text.length; i++) {
            hash ^= text.charCodeAt(i);
            hash = Math.imul(hash, 0x01000193);
        }
        return (hash >>> 0).toString(16);
    }

    // mulberry32: small seeded PRNG, so the starting positions (and so the final layout) are reproducible.
    function seededRandom(seed) {
        let state = seed >>> 0;
        return () => {
            state = (state + 0x6d2b79f5) >>> 0;
            let t = state;
            t = Math.imul(t ^ (t >>> 15), t | 1);
            t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
            return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
        };
    }

    /**
     * Split the graph into the connected tables and the isolated ones, and list one undirected link per table pair.
     *
     * @param {{nodes: Object[], edges: Object[]}} graph The schema graph.
     * @returns {{connected: string[], isolated: string[], links: Array<[string, string]>}} The split.
     */
    function splitGraph(graph) {
        const linked = new Set();
        const pairs = new Map();
        for (const edge of graph.edges) {
            if (edge.from === edge.to) {
                continue;
            }
            const [a, b] = edge.from < edge.to ? [edge.from, edge.to] : [edge.to, edge.from];
            pairs.set(`${a}\u0000${b}`, [a, b]);
            linked.add(a);
            linked.add(b);
        }
        const ids = graph.nodes.map((node) => node.id);
        return {
            connected: ids.filter((id) => linked.has(id)),
            isolated: ids.filter((id) => !linked.has(id)).sort(),
            links: [...pairs.values()],
        };
    }

    /**
     * Get a signature of a graph's tables and links, to tell whether a saved layout still matches it.
     *
     * @param {{nodes: Object[], edges: Object[]}} graph The schema graph.
     * @returns {string} The signature.
     */
    function graphSignature(graph) {
        const { connected, isolated, links } = splitGraph(graph);
        return hashString([[...connected].sort().join(","), isolated.join(","), links.map((link) => link.join(">")).sort().join(",")].join("|"));
    }

    /**
     * Get the drawn radius of a table's star, from its number of linked tables. Matches the explorer's node style.
     *
     * @param {number} degree The number of linked tables.
     * @returns {number} The radius.
     */
    function starRadius(degree) {
        return (7 + (Math.min(degree, 120) / 120) * 27) / 2;
    }

    function median(values) {
        if (values.length === 0) {
            return 0;
        }
        const sorted = [...values].sort((a, b) => a - b);
        return sorted[Math.floor(sorted.length / 2)];
    }

    /**
     * Push overlapping stars apart until none overlap (or a pass limit is hit). Fixed stars never move.
     *
     * Stars are bucketed in a grid whose cells fit the biggest star, so each pass only compares neighbouring cells.
     *
     * @param {Array<{x: number, y: number, radius: number, fixed?: boolean}>} stars The stars, moved in place.
     * @returns {number} The number of overlapping pairs left.
     */
    function separateStars(stars) {
        if (stars.length === 0) {
            return 0;
        }
        const cellSize = 2 * Math.max(...stars.map((star) => star.radius)) + STAR_MARGIN;
        let overlapping = 0;
        for (let pass = 0; pass < MAX_SEPARATION_PASSES; pass++) {
            const grid = new Map();
            stars.forEach((star, index) => {
                const cell = `${Math.floor(star.x / cellSize)},${Math.floor(star.y / cellSize)}`;
                if (!grid.has(cell)) {
                    grid.set(cell, []);
                }
                grid.get(cell).push(index);
            });

            overlapping = 0;
            stars.forEach((star, index) => {
                const cellX = Math.floor(star.x / cellSize);
                const cellY = Math.floor(star.y / cellSize);
                for (let dx = -1; dx <= 1; dx++) {
                    for (let dy = -1; dy <= 1; dy++) {
                        for (const otherIndex of grid.get(`${cellX + dx},${cellY + dy}`) || []) {
                            if (otherIndex <= index) {
                                continue;
                            }
                            const other = stars[otherIndex];
                            const minDistance = star.radius + other.radius + STAR_MARGIN;
                            let offsetX = other.x - star.x;
                            let offsetY = other.y - star.y;
                            let distance = Math.hypot(offsetX, offsetY);
                            if (distance >= minDistance - DISTANCE_TOLERANCE || (star.fixed && other.fixed)) {
                                continue;
                            }

                            // Stars on the exact same spot get a fixed direction, so the result stays deterministic.
                            if (distance === 0) {
                                [offsetX, offsetY, distance] = [1, 0, 1];
                            }

                            // Split the push between both stars, or give all of it to the one that can move.
                            const overlap = (minDistance - distance) / distance;
                            const starShare = star.fixed ? 0 : other.fixed ? 1 : 0.5;
                            star.x -= offsetX * overlap * starShare;
                            star.y -= offsetY * overlap * starShare;
                            other.x += offsetX * overlap * (1 - starShare);
                            other.y += offsetY * overlap * (1 - starShare);
                            overlapping++;
                        }
                    }
                }
            });
            if (overlapping === 0) {
                break;
            }
        }
        return overlapping;
    }

    function boundsOf(points) {
        if (points.length === 0) {
            return null;
        }
        const bounds = { minX: Infinity, minY: Infinity, maxX: -Infinity, maxY: -Infinity };
        for (const { x, y } of points) {
            bounds.minX = Math.min(bounds.minX, x);
            bounds.minY = Math.min(bounds.minY, y);
            bounds.maxX = Math.max(bounds.maxX, x);
            bounds.maxY = Math.max(bounds.maxY, y);
        }
        return bounds;
    }

    /**
     * Get a slot of the sunflower spiral holding the tables without references.
     *
     * @param {{x: number, y: number}} centre The spiral centre.
     * @param {number} slot The slot index, 0 being the innermost.
     * @returns {{x: number, y: number}} The slot position.
     */
    function spiralSlot(centre, slot) {
        const radius = SPIRAL_STEP * Math.sqrt(slot + 0.5);
        const angle = slot * GOLDEN_ANGLE;
        return { x: Math.round((centre.x + radius * Math.cos(angle)) * 10) / 10, y: Math.round((centre.y + radius * Math.sin(angle)) * 10) / 10 };
    }

    /**
     * Place tables without references on a sunflower spiral beside the graph.
     *
     * Tables already on the previous spiral (and pinned ones) keep their spot. New ones take the next free slots outward.
     *
     * @param {string[]} isolated The tables without references.
     * @param {Object<string, {x: number, y: number}>} positions The positions, filled in place.
     * @param {Object|null} previous The previous layout, if any.
     * @param {Object|null} graphBounds The bounds of the connected tables, to put a new spiral beside them.
     * @param {Set<string>} pinned The tables the user moved.
     * @returns {{x: number, y: number}|null} The spiral centre, or null if there are no tables without references.
     */
    function placeIsolated(isolated, positions, previous, graphBounds, pinned) {
        if (isolated.length === 0) {
            return null;
        }
        const hasSpiral = Boolean(previous?.clusterCentre);
        const kept = isolated.filter((id) => previous?.isolated.includes(id) && previous.positions[id] && (hasSpiral || pinned.has(id)));
        for (const id of kept) {
            positions[id] = { ...previous.positions[id] };
        }

        let centre = previous?.clusterCentre;
        if (!centre) {
            const radius = SPIRAL_STEP * Math.sqrt(isolated.length);
            centre = graphBounds ? { x: graphBounds.maxX + CLUSTER_GAP + radius, y: (graphBounds.minY + graphBounds.maxY) / 2 } : { x: 0, y: 0 };
        }

        // Continue past the outermost slot still in use, so new tables never land on old ones.
        const usedSlots = kept.filter((id) => !pinned.has(id)).map((id) => (Math.hypot(positions[id].x - centre.x, positions[id].y - centre.y) / SPIRAL_STEP) ** 2 - 0.5);
        const firstFree = usedSlots.length > 0 ? Math.ceil(Math.max(...usedSlots)) + 1 : 0;
        isolated.filter((id) => !kept.includes(id)).forEach((id, index) => {
            positions[id] = spiralSlot(centre, firstFree + index);
        });
        return { ...centre };
    }

    // Give tables without a saved position a start next to their already placed neighbours.
    // Repeated so chains of new tables get placed too; anything left starts near the middle.
    function seedNewNodes(forceGraph, newIds, random) {
        const placed = new Set(forceGraph.filterNodes((id) => !newIds.has(id)));
        const middle = boundsOf(forceGraph.filterNodes((id) => placed.has(id)).map((id) => forceGraph.getNodeAttributes(id)));
        const centre = middle ? { x: (middle.minX + middle.maxX) / 2, y: (middle.minY + middle.maxY) / 2 } : { x: 0, y: 0 };
        let pending = [...newIds];
        while (pending.length > 0) {
            const stillPending = [];
            for (const id of pending) {
                const anchors = forceGraph.neighbors(id).filter((neighbour) => placed.has(neighbour)).map((neighbour) => forceGraph.getNodeAttributes(neighbour));
                if (anchors.length === 0) {
                    stillPending.push(id);
                    continue;
                }
                const x = anchors.reduce((sum, anchor) => sum + anchor.x, 0) / anchors.length;
                const y = anchors.reduce((sum, anchor) => sum + anchor.y, 0) / anchors.length;
                forceGraph.mergeNodeAttributes(id, { x: x + (random() - 0.5), y: y + (random() - 0.5) });
                placed.add(id);
            }
            if (stillPending.length === pending.length) {
                for (const id of stillPending) {
                    forceGraph.mergeNodeAttributes(id, { x: centre.x + (random() - 0.5) * 10, y: centre.y + (random() - 0.5) * 10 });
                }
                break;
            }
            pending = stillPending;
        }
    }

    function linkLengths(forceGraph, links) {
        return links.map(([a, b]) => {
            const pa = forceGraph.getNodeAttributes(a);
            const pb = forceGraph.getNodeAttributes(b);
            return Math.hypot(pa.x - pb.x, pa.y - pb.y);
        });
    }

    // Run ForceAtlas2 in chunks until stars barely move, or stop improving. Yields between chunks to keep pages responsive.
    async function runUntilConverged(forceGraph, links, onProgress) {
        const forceAtlas2 = library.layoutForceAtlas2;

        // Dissuading hubs keeps very linked tables like `factions` from swallowing their neighbours.
        // Strong gravity keeps small disconnected groups near the rest instead of drifting far away.
        const settings = {
            ...forceAtlas2.inferSettings(forceGraph),
            barnesHutOptimize: forceGraph.order > 500,
            barnesHutTheta: 1,
            outboundAttractionDistribution: true,
            strongGravityMode: true,
            gravity: 0.05,
            scalingRatio: 10,
        };
        const movable = forceGraph.filterNodes((_id, attributes) => !attributes.fixed);
        const history = [];
        let iterations = 0;
        while (iterations < MAX_ITERATIONS && movable.length > 0) {
            const before = movable.map((id) => forceGraph.getNodeAttributes(id)).map(({ x, y }) => [x, y]);
            forceAtlas2.assign(forceGraph, { iterations: ITERATIONS_PER_CHUNK, settings });
            iterations += ITERATIONS_PER_CHUNK;

            const moved = movable.reduce((sum, id, index) => {
                const { x, y } = forceGraph.getNodeAttributes(id);
                return sum + Math.hypot(x - before[index][0], y - before[index][1]);
            }, 0);
            const movement = moved / movable.length / ITERATIONS_PER_CHUNK / (median(linkLengths(forceGraph, links)) || 1);
            history.push(movement);
            onProgress({ iterations, movement });
            await new Promise((resolve) => setTimeout(resolve, 0));

            const plateaued = history.length > PLATEAU_CHUNKS && movement > history[history.length - 1 - PLATEAU_CHUNKS] * (1 - PLATEAU_MIN_GAIN);
            if (movement < CONVERGED_MOVEMENT || plateaued) {
                break;
            }
        }
        return iterations;
    }

    /**
     * Compute the layout of a schema graph, from scratch or continuing from a previous layout.
     *
     * @param {{nodes: Object[], edges: Object[]}} graph The schema graph.
     * @param {{previous?: Object|null, onProgress?: (progress: {iterations: number, movement: number}) => void}} [options]
     *   `previous` is an earlier result of this function, possibly for an older version of the schema.
     * @returns {Promise<{positions: Object, isolated: string[], clusterCentre: Object|null, pinned: string[], scale: number, signature: string, iterations: number}>} The layout.
     */
    async function computeLayout(graph, { previous = null, onProgress = () => {} } = {}) {
        const { connected, isolated, links } = splitGraph(graph);
        const pinned = new Set((previous?.pinned || []).filter((id) => graph.nodes.some((node) => node.id === id)));
        const previousIsolated = new Set(previous?.isolated || []);
        const isKnown = (id) => Boolean(previous?.positions[id]) && !previousIsolated.has(id);
        const scale = previous?.scale > 0 && connected.some(isKnown) ? previous.scale : null;
        const random = seededRandom(connected.length * 31 + links.length);

        // Saved positions are in drawing units; ForceAtlas2 works in its own units, `scale` times smaller.
        const forceGraph = new graphology.UndirectedGraph();
        const newIds = new Set();
        const spread = Math.sqrt(connected.length) * 10;
        for (const id of connected) {
            if (scale && isKnown(id)) {
                const saved = previous.positions[id];
                forceGraph.addNode(id, { x: saved.x / scale, y: saved.y / scale, fixed: pinned.has(id) });
            } else {
                forceGraph.addNode(id, { x: (random() - 0.5) * spread, y: (random() - 0.5) * spread, fixed: false });
                newIds.add(id);
            }
        }
        for (const [a, b] of links) {
            forceGraph.addEdge(a, b);
        }
        if (newIds.size < connected.length) {
            seedNewNodes(forceGraph, newIds, random);
        }

        const iterations = await runUntilConverged(forceGraph, links, onProgress);

        // A fresh layout is scaled so a typical link has the same length whatever the schema size. Updates keep the old scale, so the map doesn't jump.
        const drawScale = scale || TARGET_EDGE_LENGTH / (median(linkLengths(forceGraph, links)) || 1);
        const stars = forceGraph.mapNodes((id, attributes) => ({
            id,
            x: attributes.x * drawScale,
            y: attributes.y * drawScale,
            radius: starRadius(forceGraph.degree(id)),
            fixed: attributes.fixed,
        }));
        separateStars(stars);

        // Pinned stars take their saved position as is: going through ForceAtlas2's float32 buffers isn't exact.
        const positions = {};
        for (const star of stars) {
            positions[star.id] = star.fixed
                ? { ...previous.positions[star.id] }
                : { x: Math.round(star.x * 10) / 10, y: Math.round(star.y * 10) / 10 };
        }
        const clusterCentre = placeIsolated(isolated, positions, previous, boundsOf(stars), pinned);
        return {
            positions,
            isolated,
            clusterCentre,
            pinned: [...pinned],
            scale: drawScale,
            signature: graphSignature(graph),
            iterations,
        };
    }

    //-------------------------------------------------------------------//
    // localStorage cache
    //-------------------------------------------------------------------//

    // localStorage can be missing or throw (private windows, file:// in some browsers), so the cache is best-effort.
    function storage() {
        try {
            return typeof window !== "undefined" ? window.localStorage : null;
        } catch (error) {
            return null;
        }
    }

    function readCache(sourceKey) {
        try {
            const cached = JSON.parse(storage()?.getItem(CACHE_PREFIX + sourceKey) || "null");
            const valid = cached && typeof cached.positions === "object" && Array.isArray(cached.isolated)
                && Array.isArray(cached.pinned) && cached.scale > 0 && typeof cached.signature === "string";
            return valid ? cached : null;
        } catch (error) {
            return null;
        }
    }

    function writeCache(sourceKey, layout) {
        try {
            storage()?.setItem(CACHE_PREFIX + sourceKey, JSON.stringify(layout));
        } catch (error) {
            console.debug("Couldn't save the layout:", error.message);
        }
    }

    /**
     * Delete layouts saved by older versions of the explorer.
     */
    function clearLegacyCaches() {
        const store = storage();
        if (!store) {
            return;
        }
        try {
            const keys = Array.from({ length: store.length }, (_, index) => store.key(index));
            for (const key of keys) {
                if (key && LEGACY_CACHE_PREFIXES.some((prefix) => key.startsWith(prefix))) {
                    store.removeItem(key);
                }
            }
        } catch (error) {
            console.debug("Couldn't clear old layouts:", error.message);
        }
    }

    /**
     * Get the layout of a schema source: the saved one if it matches the graph, otherwise computed and saved.
     *
     * If a saved layout exists for an older version of the schema, the new layout continues from it.
     *
     * @param {string} sourceKey What the schema is, e.g. `api:warhammer_3` or `file:schema_wh3.ron`.
     * @param {{nodes: Object[], edges: Object[]}} graph The schema graph.
     * @param {(progress: {iterations: number, movement: number}) => void} [onProgress] Called while computing.
     * @returns {Promise<Object>} The layout, plus `status`: `cached`, `updated` (continued from an older layout) or `new`.
     */
    async function layoutFor(sourceKey, graph, onProgress) {
        clearLegacyCaches();
        const cached = readCache(sourceKey);
        // Layouts saved before the spiral cluster have no `clusterCentre`; update them so their isolated tables move onto one.
        if (cached && cached.signature === graphSignature(graph) && cached.clusterCentre !== undefined) {
            return { ...cached, status: "cached" };
        }

        const layout = await computeLayout(graph, { previous: cached, onProgress });
        writeCache(sourceKey, layout);
        return { ...layout, status: cached ? "updated" : "new" };
    }

    /**
     * Save stars the user moved into a source's saved layout.
     *
     * @param {string} sourceKey The schema source, as given to `layoutFor`.
     * @param {Object<string, {x: number, y: number}>} moves The new positions of the moved tables.
     * @param {{pin?: boolean, clusterCentre?: {x: number, y: number}}} [options] `pin` (default true) pins the moved stars so
     *   later layouts leave them in place. `clusterCentre` saves the new centre of the spiral when the whole cluster moved.
     * @returns {boolean} Whether the layout was saved.
     */
    function saveMovedStars(sourceKey, moves, { pin = true, clusterCentre = null } = {}) {
        const cached = readCache(sourceKey);
        if (!cached) {
            return false;
        }
        const pinned = new Set(cached.pinned);
        for (const [id, { x, y }] of Object.entries(moves)) {
            if (cached.positions[id]) {
                cached.positions[id] = { x: Math.round(x * 10) / 10, y: Math.round(y * 10) / 10 };
                if (pin) {
                    pinned.add(id);
                }
            }
        }
        if (clusterCentre) {
            cached.clusterCentre = { x: Math.round(clusterCentre.x * 10) / 10, y: Math.round(clusterCentre.y * 10) / 10 };
        }
        cached.pinned = [...pinned];
        writeCache(sourceKey, cached);
        return true;
    }

    return { computeLayout, layoutFor, saveMovedStars, clearLegacyCaches, splitGraph, graphSignature, starRadius, separateStars, spiralSlot };
});
