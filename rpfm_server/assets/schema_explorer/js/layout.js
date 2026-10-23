// Precomputed ForceAtlas2 layout of the whole schema graph.
//
// The layout runs once per schema (deterministic: same schema, same picture) and the positions are
// cached in localStorage, so reopening a schema shows the full map straight away. Tables without
// any reference are left out of the force layout and packed into a grid cluster beside the graph.

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

    const CACHE_PREFIX = "rpfm-schema-explorer-layout:";
    const LAYOUT_VERSION = 2;
    const TOTAL_ITERATIONS = 300;
    const ITERATIONS_PER_CHUNK = 25;
    const MAX_SEPARATION_PASSES = 50;
    const STAR_MARGIN = 3;
    const TARGET_EDGE_LENGTH = 70;
    const CLUSTER_SPACING = 45;
    const CLUSTER_GAP = 250;

    // FNV-1a, enough to tell schemas (and layout settings) apart in the cache key.
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
     * Build the cache key of a graph's layout.
     *
     * @param {{nodes: Object[], edges: Object[]}} graph The schema graph.
     * @returns {string} The key.
     */
    function cacheKey(graph) {
        const { connected, isolated, links } = splitGraph(graph);
        const signature = [LAYOUT_VERSION, connected.join(","), isolated.join(","), links.map((link) => link.join(">")).sort().join(",")].join("|");
        return CACHE_PREFIX + hashString(signature);
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

    /**
     * Push overlapping stars apart until none overlap (or a pass limit is hit).
     *
     * Stars are bucketed in a grid whose cells fit the biggest star, so each pass only compares neighbouring cells.
     *
     * @param {Array<{x: number, y: number, radius: number}>} stars The stars, moved in place.
     * @returns {number} The number of overlapping pairs left.
     */
    function separateStars(stars) {
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
                            if (distance >= minDistance) {
                                continue;
                            }

                            // Stars on the exact same spot get a fixed direction, so the result stays deterministic.
                            if (distance === 0) {
                                [offsetX, offsetY, distance] = [1, 0, 1];
                            }
                            const push = (minDistance - distance) / 2 / distance;
                            star.x -= offsetX * push;
                            star.y -= offsetY * push;
                            other.x += offsetX * push;
                            other.y += offsetY * push;
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

    function median(values) {
        if (values.length === 0) {
            return 0;
        }
        const sorted = [...values].sort((a, b) => a - b);
        return sorted[Math.floor(sorted.length / 2)];
    }

    // Place isolated tables on a grid to the right of the main graph, vertically centred on it.
    function placeCluster(ids, positions, bounds) {
        const columns = Math.max(1, Math.ceil(Math.sqrt(ids.length)));
        const rows = Math.ceil(ids.length / columns);
        const left = bounds ? bounds.maxX + CLUSTER_GAP : 0;
        const top = bounds ? (bounds.minY + bounds.maxY) / 2 - ((rows - 1) * CLUSTER_SPACING) / 2 : 0;
        ids.forEach((id, index) => {
            positions[id] = { x: left + (index % columns) * CLUSTER_SPACING, y: top + Math.floor(index / columns) * CLUSTER_SPACING };
        });
    }

    function boundsOf(positions, ids) {
        if (ids.length === 0) {
            return null;
        }
        const bounds = { minX: Infinity, minY: Infinity, maxX: -Infinity, maxY: -Infinity };
        for (const id of ids) {
            const { x, y } = positions[id];
            bounds.minX = Math.min(bounds.minX, x);
            bounds.minY = Math.min(bounds.minY, y);
            bounds.maxX = Math.max(bounds.maxX, x);
            bounds.maxY = Math.max(bounds.maxY, y);
        }
        return bounds;
    }

    /**
     * Compute the layout of a schema graph with ForceAtlas2.
     *
     * Yields to the event loop between chunks of iterations, so a page stays responsive.
     *
     * @param {{nodes: Object[], edges: Object[]}} graph The schema graph.
     * @param {(progress: number) => void} [onProgress] Called with a 0..1 progress value.
     * @returns {Promise<{positions: Object<string, {x: number, y: number}>, isolated: string[]}>} The positions of every table, and the isolated ones.
     */
    async function computeLayout(graph, onProgress = () => {}) {
        const { connected, isolated, links } = splitGraph(graph);
        const forceGraph = new graphology.UndirectedGraph();
        const random = seededRandom(connected.length * 31 + links.length);
        const spread = Math.sqrt(connected.length) * 10;
        for (const id of connected) {
            forceGraph.addNode(id, { x: (random() - 0.5) * spread, y: (random() - 0.5) * spread, size: 1 });
        }
        for (const [a, b] of links) {
            forceGraph.addEdge(a, b);
        }

        // Dissuading hubs keeps very linked tables like `factions` from swallowing their neighbours.
        const forceAtlas2 = library.layoutForceAtlas2;
        const settings = {
            ...forceAtlas2.inferSettings(forceGraph),
            barnesHutOptimize: connected.length > 500,
            barnesHutTheta: 1,
            outboundAttractionDistribution: true,
            strongGravityMode: false,
            gravity: 1,
            scalingRatio: 10,
        };
        for (let done = 0; done < TOTAL_ITERATIONS; done += ITERATIONS_PER_CHUNK) {
            forceAtlas2.assign(forceGraph, { iterations: ITERATIONS_PER_CHUNK, settings });
            onProgress(Math.min((done + ITERATIONS_PER_CHUNK) / TOTAL_ITERATIONS, 1) * 0.95);
            await new Promise((resolve) => setTimeout(resolve, 0));
        }

        // Scale so a typical link has the same on-screen length whatever the schema size, then nudge overlapping stars apart.
        const lengths = links.map(([a, b]) => {
            const pa = forceGraph.getNodeAttributes(a);
            const pb = forceGraph.getNodeAttributes(b);
            return Math.hypot(pa.x - pb.x, pa.y - pb.y);
        });
        const scale = TARGET_EDGE_LENGTH / (median(lengths) || 1);
        const stars = forceGraph.mapNodes((id, attributes) => ({
            id,
            x: attributes.x * scale,
            y: attributes.y * scale,
            radius: starRadius(forceGraph.degree(id)),
        }));
        separateStars(stars);

        const positions = {};
        for (const star of stars) {
            positions[star.id] = { x: Math.round(star.x * 10) / 10, y: Math.round(star.y * 10) / 10 };
        }
        placeCluster(isolated, positions, boundsOf(positions, connected));
        onProgress(1);
        return { positions, isolated };
    }

    /**
     * Get a graph's layout from the cache, or compute and cache it.
     *
     * @param {{nodes: Object[], edges: Object[]}} graph The schema graph.
     * @param {(progress: number) => void} [onProgress] Called with a 0..1 progress value while computing.
     * @returns {Promise<{positions: Object, isolated: string[], fromCache: boolean}>} The layout.
     */
    async function layoutFor(graph, onProgress) {
        const key = cacheKey(graph);
        try {
            const cached = JSON.parse(window.localStorage.getItem(key) || "null");
            if (cached && cached.positions && Array.isArray(cached.isolated)) {
                return { ...cached, fromCache: true };
            }
        } catch (error) {
            console.debug("Layout cache unavailable:", error.message);
        }

        const layout = await computeLayout(graph, onProgress);
        try {
            window.localStorage.setItem(key, JSON.stringify(layout));
        } catch (error) {
            console.debug("Couldn't cache the layout:", error.message);
        }
        return { ...layout, fromCache: false };
    }

    /**
     * Forget a graph's cached layout, so the next `layoutFor` computes it again.
     *
     * @param {{nodes: Object[], edges: Object[]}} graph The schema graph.
     */
    function clearCachedLayout(graph) {
        try {
            window.localStorage.removeItem(cacheKey(graph));
        } catch (error) {
            console.debug("Couldn't clear the layout cache:", error.message);
        }
    }

    return { computeLayout, layoutFor, clearCachedLayout, splitGraph, cacheKey, starRadius, separateStars };
});
