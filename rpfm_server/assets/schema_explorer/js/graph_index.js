// Graph queries for the explorer: neighbourhoods, shortest paths and table search.
//
// Works on the `graph` JSON (nodes + edges) and has no DOM dependencies, so it's tested under Node.

(function (root, factory) {
    const api = factory();
    if (typeof module === "object" && module.exports) {
        module.exports = api;
    } else {
        root.RpfmGraphIndex = api;
    }
})(typeof self !== "undefined" ? self : this, () => {
    "use strict";

    const DIRECTIONS = new Set(["both", "out", "in"]);

    function addEdge(map, from, to, edge) {
        if (!map.has(from)) {
            map.set(from, new Map());
        }
        const targets = map.get(from);
        if (!targets.has(to)) {
            targets.set(to, []);
        }
        targets.get(to).push(edge);
    }

    class GraphIndex {

        /**
         * @param {{nodes: Object[], edges: Object[]}} graph The graph to index.
         */
        constructor(graph) {
            this.nodes = new Map(graph.nodes.map((node) => [node.id, node]));
            this.outgoing = new Map();
            this.incoming = new Map();
            for (const edge of graph.edges) {
                addEdge(this.outgoing, edge.from, edge.to, edge);
                addEdge(this.incoming, edge.to, edge.from, edge);
            }

            // Degree counts distinct neighbouring tables, not columns, so a table referenced by many columns of one table isn't a hub.
            this.degrees = new Map();
            for (const id of this.nodes.keys()) {
                const neighbours = new Set([...this.neighbours(id, "out"), ...this.neighbours(id, "in")]);
                neighbours.delete(id);
                this.degrees.set(id, neighbours.size);
            }
            this.searchEntries = [...this.nodes.values()].map((node) => ({ node, id: node.id.toLowerCase() }));
        }

        degree(id) {
            return this.degrees.get(id) || 0;
        }

        /**
         * Get the ids of a table's neighbours.
         *
         * @param {string} id The table id.
         * @param {"both"|"out"|"in"} direction `out` follows references, `in` follows "referenced by".
         * @returns {string[]} The neighbour ids.
         */
        neighbours(id, direction) {
            if (!DIRECTIONS.has(direction)) {
                throw new Error(`Unknown direction '${direction}'`);
            }
            const result = new Set();
            if (direction !== "in") {
                for (const target of this.outgoing.get(id)?.keys() || []) {
                    result.add(target);
                }
            }
            if (direction !== "out") {
                for (const source of this.incoming.get(id)?.keys() || []) {
                    result.add(source);
                }
            }
            return [...result];
        }

        /**
         * Get the tables around a focused one, breadth-first.
         *
         * Hubs (degree above `hubThreshold`) other than the focus are never expanded through.
         * With `hideHubs`, they're left out entirely.
         *
         * @param {string} focus The focused table id.
         * @param {{depth: number, direction: string, hubThreshold: number, hideHubs: boolean, maxNodes: number}} options Traversal options.
         * @returns {{ids: Set<string>, truncated: boolean}} The visible ids, and whether `maxNodes` cut the result short.
         */
        neighbourhood(focus, options) {
            const { depth, direction, hubThreshold, hideHubs, maxNodes } = options;
            const ids = new Set([focus]);
            let frontier = [focus];
            let truncated = false;
            for (let level = 0; level < depth && frontier.length > 0 && !truncated; level++) {
                const next = [];
                for (const id of frontier) {
                    if (id !== focus && this.degree(id) > hubThreshold) {
                        continue;
                    }
                    for (const neighbour of this.neighbours(id, direction)) {
                        if (ids.has(neighbour) || (hideHubs && this.degree(neighbour) > hubThreshold)) {
                            continue;
                        }
                        if (ids.size >= maxNodes) {
                            truncated = true;
                            break;
                        }
                        ids.add(neighbour);
                        next.push(neighbour);
                    }
                    if (truncated) {
                        break;
                    }
                }
                frontier = next;
            }
            return { ids, truncated };
        }

        /**
         * Find a shortest path between two tables.
         *
         * @param {string} from The start table id.
         * @param {string} to The end table id.
         * @param {{direction: string, avoidHubsOver: number|null}} options `avoidHubsOver` skips hubs as intermediate steps.
         * @returns {string[]|null} The ids along the path (including both ends), or null if there's none.
         */
        shortestPath(from, to, options) {
            if (!this.nodes.has(from) || !this.nodes.has(to)) {
                return null;
            }
            const previous = new Map([[from, null]]);
            let frontier = [from];
            while (frontier.length > 0 && !previous.has(to)) {
                const next = [];
                for (const id of frontier) {
                    for (const neighbour of this.neighbours(id, options.direction)) {
                        if (previous.has(neighbour)) {
                            continue;
                        }
                        const isBlockedHub = options.avoidHubsOver !== null && neighbour !== to && this.degree(neighbour) > options.avoidHubsOver;
                        if (isBlockedHub) {
                            continue;
                        }
                        previous.set(neighbour, id);
                        next.push(neighbour);
                    }
                }
                frontier = next;
            }
            if (!previous.has(to)) {
                return null;
            }
            const path = [];
            for (let id = to; id !== null; id = previous.get(id)) {
                path.push(id);
            }
            return path.reverse();
        }

        /**
         * Get the edges between two tables, in both directions.
         *
         * @param {string} a A table id.
         * @param {string} b Another table id.
         * @returns {Object[]} The edges.
         */
        edgesBetween(a, b) {
            const forward = this.outgoing.get(a)?.get(b) || [];
            const backward = a === b ? [] : this.outgoing.get(b)?.get(a) || [];
            return [...forward, ...backward];
        }

        /**
         * Search tables by name. Exact matches rank first, then prefixes, word prefixes, substrings and subsequences.
         *
         * @param {string} query The search text.
         * @param {number} limit The maximum number of results.
         * @returns {Object[]} The matching nodes, best first.
         */
        search(query, limit) {
            const needle = query.trim().toLowerCase().replace(/_tables$/, "");
            if (needle.length === 0) {
                return [];
            }
            const scored = [];
            for (const entry of this.searchEntries) {
                const score = matchScore(entry.id, needle);
                if (score !== null) {
                    scored.push({ node: entry.node, score });
                }
            }
            scored.sort((a, b) => a.score - b.score || a.node.id.length - b.node.id.length || a.node.id.localeCompare(b.node.id));
            return scored.slice(0, limit).map((entry) => entry.node);
        }
    }

    function matchScore(id, needle) {
        if (id === needle) {
            return 0;
        }
        if (id.startsWith(needle)) {
            return 1;
        }
        const position = id.indexOf(needle);
        if (position > 0) {
            return id[position - 1] === "_" ? 2 : 3;
        }
        let from = 0;
        for (const char of needle) {
            from = id.indexOf(char, from);
            if (from === -1) {
                return null;
            }
            from++;
        }
        return 4;
    }

    return { GraphIndex };
});
