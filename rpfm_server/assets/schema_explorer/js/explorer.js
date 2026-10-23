// Schema explorer UI: wires the data source, the graph view (Cytoscape.js) and the details panel together.
//
// The whole schema is drawn at once on a saved ForceAtlas2 layout (see layout.js). Selecting a table
// recolours it and its neighbours instead of redrawing, so the map never moves around. Dragged stars
// are saved into the layout.
// Everything shown comes from the schema (definitions and references). No table data is ever read.
// All schema strings go into the DOM through `textContent`, never `innerHTML`, as they can contain anything.

(() => {
    "use strict";

    const { WsSource } = window.RpfmDataSource;
    const { GraphIndex, pairKey } = window.RpfmGraphIndex;
    const { layoutFor, saveMovedStars } = window.RpfmLayout;

    const SPREAD_PREFIX = "rpfm-schema-explorer-spread:";
    const MIN_SPREAD = 0.1;
    const MAX_SPREAD = 10;
    const SEARCH_LIMIT = 30;
    const CLUSTER_ID = "__tables_without_references__";
    const HUB_THRESHOLD = 40;
    const MAX_FOCUS_ZOOM = 1.6;
    const EDGE_CURVATURE = 0.12;
    const MAX_EDGE_CURVE = 90;
    const CLUSTER_MARGIN = 30;
    const PANEL_GAP = 24;
    const EXPLORE_COLUMN_GAP = 320;
    const EXPLORE_ROW_GAP = 36;
    const EXPLORE_HIGHLIGHT_CLASSES = "in out both selected";

    // Graph colours are fixed (the graph always sits on the space background), unlike the page theme.
    const COLOURS = {
        star: "#ffffff",
        starGlow: "#bcd4ff",
        missing: "#7d8597",
        edge: "#8fa8ff",
        label: "#dfe8ff",
        labelOutline: "#04050d",
        selected: "#b46cff",
        incoming: "#ff4d5e",
        outgoing: "#ffd23f",
        both: "#ff9f43",
        incomingDark: "#a62b3b",
        outgoingDark: "#a8860f",
        bothDark: "#a85d17",
        path: "#4dffb8",
        search: "#4da3ff",
        cluster: "#6b7bb5",
    };

    const elements = {
        schemaLabel: document.getElementById("schema-label"),
        search: document.getElementById("search"),
        searchResults: document.getElementById("search-results"),
        graph: document.getElementById("graph"),
        exploreGraph: document.getElementById("explore-graph"),
        contextMenu: document.getElementById("context-menu"),
        contextExplore: document.getElementById("context-explore"),
        title: document.getElementById("title"),
        emptyState: document.getElementById("empty-state"),
        status: document.getElementById("graph-status"),
        details: document.getElementById("details"),
        pathBox: document.getElementById("path-box"),
        pathFrom: document.getElementById("path-from"),
        pathFromSuggestions: document.getElementById("path-from-suggestions"),
        pathTo: document.getElementById("path-to"),
        pathToSuggestions: document.getElementById("path-to-suggestions"),
        pathClear: document.getElementById("path-clear"),
        pathResult: document.getElementById("path-result"),
        spreadControl: document.getElementById("spread-control"),
        spread: document.getElementById("spread"),
        spreadValue: document.getElementById("spread-value"),
    };

    const state = {
        source: null,
        sourceKey: null,
        game: null,
        graph: null,
        index: null,
        selected: null,
        path: null,
        detailsCache: new Map(),
        isolated: new Set(),
        spread: 1,
        spreadCentre: { x: 0, y: 0 },
        clusterCentre: null,
        summary: "",
        openSections: { fields: true, localised: true, referencedBy: true },
        view: "galaxy",
        exploration: null,
        menuTarget: null,
        pairsByKey: new Map(),
    };

    const cy = window.cytoscape({
        container: elements.graph,
        style: graphStyle(),
        minZoom: 0.02,
        maxZoom: 5,
        boxSelectionEnabled: false,
    });

    // Explorer has its own graph, so it never disturbs the Galaxy layout.
    const exploreCy = window.cytoscape({
        container: elements.exploreGraph,
        style: graphStyle(),
        minZoom: 0.05,
        maxZoom: 5,
        boxSelectionEnabled: false,
    });

    //-------------------------------------------------------------------//
    // Remembered spread
    //-------------------------------------------------------------------//

    // localStorage can be missing or throw (private windows, file:// in some browsers), so this is best-effort.
    function loadSpread(sourceKey) {
        try {
            const spread = Number.parseFloat(window.localStorage.getItem(SPREAD_PREFIX + sourceKey));
            return Number.isFinite(spread) ? Math.min(Math.max(spread, MIN_SPREAD), MAX_SPREAD) : 1;
        } catch (error) {
            return 1;
        }
    }

    function saveSpread(sourceKey, spread) {
        try {
            window.localStorage.setItem(SPREAD_PREFIX + sourceKey, String(spread));
        } catch (error) {
            console.debug("Couldn't save the spread:", error.message);
        }
    }

    //-------------------------------------------------------------------//
    // Status and graph style
    //-------------------------------------------------------------------//

    function setStatus(message, isError = false) {
        elements.status.textContent = message;
        elements.status.classList.toggle("error", isError);
    }

    function edgeColour(colour) {
        return { "line-color": colour, "target-arrow-color": colour, "source-arrow-color": colour };
    }

    // Styles for a highlighted table and its neighbours, for classes named `<prefix>selected`, `<prefix>in`, `<prefix>out` and `<prefix>both`.
    function highlightRules(prefix, opacity) {
        const nodes = ["selected", "in", "out", "both"].map((kind) => `node.${prefix}${kind}`).join(", ");
        return [
            { selector: `node.${prefix}selected`, style: { "background-color": COLOURS.selected } },
            { selector: `node.${prefix}in`, style: { "background-color": COLOURS.incoming } },
            { selector: `node.${prefix}out`, style: { "background-color": COLOURS.outgoing } },
            { selector: `node.${prefix}both`, style: { "background-color": COLOURS.both } },
            { selector: nodes, style: { "opacity": opacity, "underlay-opacity": 0, "label": "data(id)", "text-opacity": 1 } },
            { selector: `edge.${prefix}in`, style: { ...edgeColour(COLOURS.incomingDark), "opacity": opacity, "width": 2, "arrow-scale": 1 } },
            { selector: `edge.${prefix}out`, style: { ...edgeColour(COLOURS.outgoingDark), "opacity": opacity, "width": 2, "arrow-scale": 1 } },
            { selector: `edge.${prefix}both`, style: { ...edgeColour(COLOURS.bothDark), "opacity": opacity, "width": 2, "arrow-scale": 1 } },
        ];
    }

    function graphStyle() {
        return [
            {
                selector: "node",
                style: {
                    "shape": "star",
                    "background-color": COLOURS.star,
                    "underlay-color": COLOURS.starGlow,
                    "underlay-opacity": 0.16,
                    "underlay-padding": 3,
                    "underlay-shape": "ellipse",
                    "overlay-opacity": 0,
                    "z-index": 10,
                    "color": COLOURS.label,
                    "font-size": 9,
                    "text-valign": "bottom",
                    "text-margin-y": 2,
                    "text-outline-color": COLOURS.labelOutline,
                    "text-outline-width": 2,
                },
            },
            { selector: "node[degree]", style: { "width": "mapData(degree, 0, 120, 7, 34)", "height": "mapData(degree, 0, 120, 7, 34)" } },
            { selector: "node.hub", style: { "underlay-opacity": 0.26, "underlay-padding": 4 } },
            { selector: "node.missing", style: { "background-color": COLOURS.missing, "underlay-opacity": 0.05 } },
            {
                selector: `node#${CLUSTER_ID}`,
                style: {
                    "shape": "ellipse",
                    "width": "data(diameter)",
                    "height": "data(diameter)",
                    "z-index": 0,
                    "background-color": COLOURS.cluster,
                    "background-opacity": 0.08,
                    "border-color": COLOURS.cluster,
                    "border-style": "dashed",
                    "border-width": 1,
                    "underlay-opacity": 0,
                    "label": "data(label)",
                    "text-valign": "top",
                    "text-halign": "center",
                    "text-margin-y": -6,
                    "font-size": 16,
                    "min-zoomed-font-size": 0,
                    "color": COLOURS.label,
                },
            },
            {
                selector: "edge",
                style: {
                    "curve-style": "unbundled-bezier",
                    "control-point-distances": "data(curve)",
                    "control-point-weights": 0.5,
                    "width": 1,
                    ...edgeColour(COLOURS.edge),
                    "opacity": 0.3,
                    "overlay-opacity": 0,
                    "target-arrow-shape": "triangle",
                    "arrow-scale": 0.6,
                    "font-size": 10,
                    "color": COLOURS.label,
                    "text-outline-color": COLOURS.labelOutline,
                    "text-outline-width": 2,
                    "text-wrap": "wrap",
                    "text-rotation": "autorotate",
                },
            },
            { selector: "edge.bidirectional", style: { "source-arrow-shape": "triangle" } },
            { selector: "edge.loop", style: { "curve-style": "bezier", "loop-direction": "-45deg", "loop-sweep": "-60deg" } },

            // Highlighting only recolours stars (no halo), and each edge takes a darker shade of its neighbour's colour.
            // Hovering previews the same colours at half opacity. Later rules win, so a selection overrides a preview.
            { selector: "node.dim", style: { "opacity": 0.18, "text-opacity": 0, "underlay-opacity": 0 } },
            { selector: "edge.dim", style: { "opacity": 0.03 } },
            ...highlightRules("hover-", 0.5),
            ...highlightRules("", 1),
            { selector: "node.path", style: { "background-color": COLOURS.path, "opacity": 1, "underlay-opacity": 0 } },
            { selector: "edge.path", style: { ...edgeColour(COLOURS.path), "opacity": 1, "width": 2.5, "arrow-scale": 1 } },
            { selector: "node.labelled", style: { "label": "data(id)", "text-opacity": 1 } },

            // Search results show in blue over everything else while the search box is in use, without touching the selection.
            { selector: "node.search-match", style: { "background-color": COLOURS.search, "opacity": 1, "underlay-opacity": 0, "label": "data(id)", "text-opacity": 1 } },
            { selector: "edge:selected", style: { "label": "data(label)", "opacity": 1 } },
        ];
    }

    //-------------------------------------------------------------------//
    // Loading data
    //-------------------------------------------------------------------//

    async function useSource(source) {
        state.source = source;
        state.detailsCache.clear();
        elements.schemaLabel.textContent = `RPFM: ${source.label}`;
        elements.schemaLabel.hidden = false;
        await loadGame(source.key);
    }

    // Fetch the game's schema from the rpfm_server that served this page.
    async function loadFromServer(game) {
        setStatus(`Loading the ${game || "default game's"} schema from RPFM...`);
        try {
            await useSource(await WsSource.load(game));
        } catch (error) {
            setStatus(`Couldn't load the schema from RPFM: ${error.message}`, true);
        }
    }

    async function loadGame(game) {
        setStatus("Loading schema graph...");
        hideDetails();
        let layout;
        try {
            const graph = await state.source.graph(game);
            state.game = game;
            state.sourceKey = `ws:${game}`;
            state.graph = graph;
            state.index = new GraphIndex(graph);
            state.detailsCache.clear();
            layout = await layoutFor(state.sourceKey, graph, ({ iterations, movement }) => {
                setStatus(`Laying out the schema until it settles: ${iterations} iterations, movement ${movement.toFixed(4)}...`);
            });
        } catch (error) {
            setStatus(`Couldn't load the schema graph: ${error.message}`, true);
            return;
        }

        elements.emptyState.hidden = true;
        elements.search.disabled = false;
        elements.pathBox.hidden = false;
        state.exploration = null;
        exploreCy.elements().remove();
        state.view = "galaxy";
        elements.title.textContent = "Galaxy";
        elements.exploreGraph.hidden = true;
        elements.graph.hidden = false;
        cy.resize();

        state.spread = loadSpread(state.sourceKey);
        showSpread();
        renderOverview(layout);
        elements.spreadControl.hidden = false;
        highlightSearchMatches(state.index.search(elements.search.value, SEARCH_LIMIT));
        const tableCount = state.graph.nodes.filter((node) => !node.missing).length;
        const layoutNote = {
            cached: "saved layout",
            updated: `layout updated for schema changes in ${layout.iterations} iterations`,
            new: `new layout, settled in ${layout.iterations} iterations`,
        }[layout.status];
        state.summary = `${tableCount} tables, ${state.graph.edges.length} references, ${layout.isolated.length} without references (${layoutNote}). Search or click a star.`;
        setStatus(state.summary);

        const hashTable = readHash().table;
        if (hashTable && state.index.nodes.has(hashTable)) {
            selectTable(hashTable, { zoom: true });
        }
    }

    //-------------------------------------------------------------------//
    // Graph view
    //-------------------------------------------------------------------//

    // Curve every edge by a fraction of its length, so long edges arc as much as short ones.
    function edgeCurve(a, b) {
        return Math.min(Math.hypot(a.x - b.x, a.y - b.y) * EDGE_CURVATURE, MAX_EDGE_CURVE);
    }

    function pairLabel(pair) {
        return [...pair.forward, ...pair.backward].map((edge) => `${edge.from}.${edge.from_column} -> ${edge.to}.${edge.to_column}`).join("\n");
    }

    // Circle around some points: centred on their bounds, reaching the farthest one plus a margin.
    function boundingCircle(points, margin) {
        const xs = points.map((point) => point.x);
        const ys = points.map((point) => point.y);
        const centre = { x: (Math.min(...xs) + Math.max(...xs)) / 2, y: (Math.min(...ys) + Math.max(...ys)) / 2 };
        const radius = Math.max(...points.map((point) => Math.hypot(point.x - centre.x, point.y - centre.y))) + margin;
        return { centre, radius };
    }

    function isolatedNodes() {
        return cy.nodes().filter((node) => state.isolated.has(node.id()));
    }

    // Fit the cluster circle around the tables without references, after one of them moved.
    function updateClusterCircle() {
        const stars = isolatedNodes();
        if (stars.length === 0) {
            return;
        }
        const { centre, radius } = boundingCircle(stars.map((star) => star.position()), CLUSTER_MARGIN);
        cy.getElementById(CLUSTER_ID).position(centre).data("diameter", radius * 2);
    }

    // One edge per pair of tables: pointing the way the references go, or both ways when they go both ways.
    function pairEdgeItem(pair, positions) {
        const hasForward = pair.forward.length > 0;
        const hasBackward = pair.backward.length > 0;
        const [source, target] = hasForward ? [pair.a, pair.b] : [pair.b, pair.a];
        const classes = [];
        if (pair.a === pair.b) {
            classes.push("loop");
        } else if (hasForward && hasBackward) {
            classes.push("bidirectional");
        }
        return {
            group: "edges",
            data: {
                id: pair.key,
                source,
                target,
                label: pairLabel(pair),
                curve: pair.a === pair.b ? 0 : edgeCurve(positions[source], positions[target]),
            },
            classes: classes.join(" "),
        };
    }

    function starItem(id, position, extraClasses = []) {
        const classes = [...extraClasses];
        if (state.index.nodes.get(id)?.missing) {
            classes.push("missing");
        }
        if (state.index.degree(id) > HUB_THRESHOLD) {
            classes.push("hub");
        }
        return { group: "nodes", data: { id, degree: state.index.degree(id) }, position: { ...position }, classes: classes.join(" ") };
    }

    function renderOverview(layout) {
        const isolated = new Set(layout.isolated);
        state.isolated = isolated;
        state.clusterCentre = layout.clusterCentre ? { ...layout.clusterCentre } : null;
        state.spreadCentre = boundingCircle(Object.values(layout.positions), 0).centre;
        const positions = {};
        for (const [id, position] of Object.entries(layout.positions)) {
            positions[id] = toDisplay(position);
        }
        const items = [];
        if (isolated.size > 0) {
            const { centre, radius } = boundingCircle(layout.isolated.map((id) => positions[id]), CLUSTER_MARGIN);
            items.push({
                group: "nodes",
                data: { id: CLUSTER_ID, label: `Tables without references (${isolated.size})`, diameter: radius * 2 },
                position: centre,
                selectable: false,
            });
        }

        for (const node of state.graph.nodes) {
            items.push(starItem(node.id, positions[node.id]));
        }

        state.pairsByKey = new Map(state.index.linkedPairs().map((pair) => [pair.key, pair]));
        for (const pair of state.pairsByKey.values()) {
            items.push(pairEdgeItem(pair, positions));
        }

        state.selected = null;
        state.path = null;
        cy.batch(() => {
            cy.elements().remove();
            cy.add(items);
        });
        updateMinZoom();
        fitTo(cy.nodes(), { maxZoom: Infinity, animate: false });
    }

    function clearHighlight() {
        cy.batch(() => cy.elements().removeClass("dim in out both path selected labelled"));
    }

    function clearSelection() {
        state.selected = null;
        state.path = null;
        resetPathResult();
        clearHighlight();
        exploreCy.batch(() => exploreCy.elements().removeClass(EXPLORE_HIGHLIGHT_CLASSES));
        cy.$(":selected").unselect();
        hideDetails();
        writeHash();
        setStatus(state.view === "explore" ? exploreStatus() : state.summary);
    }

    // Fit the view to some nodes from their positions alone. Cytoscape's own fit measures every label,
    // which takes seconds on a whole schema.
    function fitTo(nodes, { maxZoom, animate }) {
        if (nodes.length === 0) {
            return;
        }
        const view = nodes.cy();
        const extent = nodes.boundingBox({ includeLabels: false, includeOverlays: false, includeUnderlays: false });
        const padding = 50;
        const covered = elements.details.hidden ? 0 : elements.details.offsetWidth + PANEL_GAP;
        const width = Math.max(view.width() - covered, 2 * padding + 1);
        const fitZoom = Math.min(
            maxZoom,
            view.maxZoom(),
            (width - 2 * padding) / Math.max(extent.w, 1),
            (view.height() - 2 * padding) / Math.max(extent.h, 1),
        );
        const zoom = Math.max(fitZoom, view.minZoom());
        const pan = { x: width / 2 - zoom * (extent.x1 + extent.w / 2), y: view.height() / 2 - zoom * (extent.y1 + extent.h / 2) };
        view.stop();
        if (animate) {
            view.animate({ zoom, pan }, { duration: 450 });
        } else {
            view.viewport({ zoom, pan });
        }
    }

    // Zoom to some tables without zooming in so far that one table fills the screen.
    function zoomTo(nodes) {
        fitTo(nodes, { maxZoom: MAX_FOCUS_ZOOM, animate: true });
    }

    // Never zoom out past the point where the view spans the diameter of the circle around all stars.
    function updateMinZoom() {
        const stars = cy.nodes().not(`#${CLUSTER_ID}`);
        if (stars.length === 0) {
            return;
        }
        const { radius } = boundingCircle(stars.map((star) => star.position()), 0);
        const minZoom = Math.min(cy.width(), cy.height()) / Math.max(radius * 2, 1);
        cy.minZoom(Math.min(minZoom, cy.maxZoom()));
        if (cy.zoom() < cy.minZoom()) {
            cy.zoom({ level: cy.minZoom(), renderedPosition: { x: cy.width() / 2, y: cy.height() / 2 } });
        }
    }

    // Classify a neighbour of the selected table: `in` references it, `out` is referenced by it, `both` is both.
    function relation(selected, neighbour) {
        const references = state.index.edgesBetween(selected, neighbour);
        const out = references.some((edge) => edge.from === selected && edge.to === neighbour);
        const incoming = references.some((edge) => edge.from === neighbour && edge.to === selected);
        if (out && incoming) {
            return "both";
        }
        return out ? "out" : "in";
    }

    // Give a table, its neighbours and the edges between them their highlight classes, e.g. `selected`, `in`, `out`, `both`.
    function colourNeighbours(view, id, prefix) {
        const kinds = state.index.neighbours(id, "both").filter((neighbour) => neighbour !== id).map((neighbour) => [neighbour, relation(id, neighbour)]);
        view.getElementById(id).addClass(`${prefix}selected`);
        for (const [neighbour, kind] of kinds) {
            view.getElementById(neighbour).addClass(`${prefix}${kind}`);
            view.getElementById(pairKey(id, neighbour)).addClass(`${prefix}${kind}`);
        }
        view.getElementById(pairKey(id, id)).addClass(`${prefix}out`);
        return kinds;
    }

    function highlightNeighbours(id) {
        const counts = { in: 0, out: 0, both: 0 };
        let kinds = [];
        cy.batch(() => {
            clearHighlight();
            kinds = colourNeighbours(cy, id, "");
            const lit = cy.collection([id, ...kinds.map(([neighbour]) => neighbour)].map((litId) => cy.getElementById(litId)));
            const kept = lit.union(cy.getElementById(id).connectedEdges()).union(cy.getElementById(CLUSTER_ID));
            cy.elements().difference(kept).addClass("dim");
            lit.addClass("labelled");
        });
        for (const [, kind] of kinds) {
            counts[kind]++;
        }
        return { ids: new Set([id, ...kinds.map(([neighbour]) => neighbour)]), counts };
    }

    const PREVIEW_CLASSES = "hover-selected hover-in hover-out hover-both";

    function clearPreview(view) {
        view.batch(() => view.$(".hover-selected, .hover-in, .hover-out, .hover-both").removeClass(PREVIEW_CLASSES));
    }

    function previewNeighbours(view, id) {
        view.batch(() => {
            view.$(".hover-selected, .hover-in, .hover-out, .hover-both").removeClass(PREVIEW_CLASSES);
            colourNeighbours(view, id, "hover-");
        });
    }

    function highlightPath(path) {
        const onPath = new Set(path);
        cy.batch(() => {
            clearHighlight();
            cy.elements().addClass("dim");
            for (const id of path) {
                cy.getElementById(id).removeClass("dim").addClass("path labelled");
            }
            for (let i = 0; i + 1 < path.length; i++) {
                cy.getElementById(pairKey(path[i], path[i + 1])).removeClass("dim").addClass("path");
            }
            cy.getElementById(path[0]).removeClass("path").addClass("selected");
            cy.getElementById(CLUSTER_ID).removeClass("dim");
        });
        zoomTo(cy.nodes().filter((node) => onPath.has(node.id())));
    }

    function selectTable(id, options = {}) {
        if (!state.index?.nodes.has(id)) {
            return;
        }
        state.selected = id;
        state.path = null;
        if (document.activeElement !== elements.pathFrom) {
            elements.pathFrom.value = id;
        }
        const { ids, counts } = highlightNeighbours(id);
        cy.$(":selected").unselect();
        showDetails(id);
        if (options.zoom) {
            zoomTo(cy.nodes().filter((node) => ids.has(node.id())));
        }
        writeHash();
        setStatus(`${id}: references ${counts.out + counts.both} tables, referenced by ${counts.in + counts.both}.`);
    }

    // Save dragged stars into the layout, and re-curve their edges for the new distances.
    // Moving the whole cluster moves the spiral (its stars aren't pinned); moving one star pins it.
    function saveDrag(node) {
        const isCluster = node.id() === CLUSTER_ID;
        const moved = isCluster ? isolatedNodes() : node;
        const moves = {};
        for (const star of moved) {
            moves[star.id()] = toLayout(star.position());
        }
        cy.batch(() => {
            for (const edge of moved.connectedEdges()) {
                if (!edge.isLoop()) {
                    edge.data("curve", edgeCurve(edge.source().position(), edge.target().position()));
                }
            }
        });
        if (state.isolated.has(node.id())) {
            updateClusterCircle();
        }
        const options = isCluster ? { pin: false, clusterCentre: state.clusterCentre } : {};
        if (!saveMovedStars(state.sourceKey, moves, options)) {
            setStatus("Couldn't save the new position: this browser doesn't keep data for this page.", true);
        }
    }

    //-------------------------------------------------------------------//
    // Spread: the drawn layout is the saved one scaled around its centre.
    //-------------------------------------------------------------------//

    function toDisplay(position) {
        const centre = state.spreadCentre;
        return { x: centre.x + (position.x - centre.x) * state.spread, y: centre.y + (position.y - centre.y) * state.spread };
    }

    function toLayout(position) {
        const centre = state.spreadCentre;
        return { x: centre.x + (position.x - centre.x) / state.spread, y: centre.y + (position.y - centre.y) / state.spread };
    }

    // The slider holds log10 of the spread, so 0.1x-1x gets as much travel as 1x-10x.
    function showSpread() {
        elements.spread.value = String(Math.log10(state.spread));
        elements.spreadValue.textContent = `${state.spread.toFixed(2)}\u00d7`;
    }

    // Move every star by the ratio between the new and the current spread, keeping dragged stars in place relative to the rest.
    function applySpread(spread) {
        const ratio = spread / state.spread;
        const centre = state.spreadCentre;
        state.spread = spread;
        showSpread();
        cy.batch(() => {
            for (const star of cy.nodes().not(`#${CLUSTER_ID}`)) {
                const position = star.position();
                star.position({ x: centre.x + (position.x - centre.x) * ratio, y: centre.y + (position.y - centre.y) * ratio });
            }
            for (const edge of cy.edges()) {
                if (!edge.isLoop()) {
                    edge.data("curve", edgeCurve(edge.source().position(), edge.target().position()));
                }
            }
        });
        updateClusterCircle();
        updateMinZoom();
    }

    // Apply slider moves at most once per frame, and remember the value once the user lets go.
    let pendingSpread = null;
    elements.spread.addEventListener("input", () => {
        const scheduled = pendingSpread !== null;
        pendingSpread = 10 ** Number.parseFloat(elements.spread.value);
        if (!scheduled) {
            requestAnimationFrame(() => {
                applySpread(pendingSpread);
                pendingSpread = null;
            });
        }
    });
    elements.spread.addEventListener("change", () => saveSpread(state.sourceKey, state.spread));

    // Dragging the cluster circle carries its stars (and the spiral centre) along.
    let clusterDragFrom = null;
    cy.on("grab", `#${CLUSTER_ID}`, (event) => {
        clusterDragFrom = { ...event.target.position() };
    });
    cy.on("drag", `#${CLUSTER_ID}`, (event) => {
        const position = event.target.position();
        const delta = { x: position.x - clusterDragFrom.x, y: position.y - clusterDragFrom.y };
        isolatedNodes().shift(delta);
        if (state.clusterCentre) {
            state.clusterCentre = { x: state.clusterCentre.x + delta.x / state.spread, y: state.clusterCentre.y + delta.y / state.spread };
        }
        clusterDragFrom = { ...position };
    });

    cy.on("tap", "node", (event) => {
        if (event.target.id() !== CLUSTER_ID) {
            selectTable(event.target.id());
        } else if (state.selected) {
            clearSelection();
        }
    });
    cy.on("dbltap", "node", (event) => {
        if (event.target.id() !== CLUSTER_ID) {
            selectTable(event.target.id(), { zoom: true });
        }
    });
    cy.on("tap", (event) => {
        if (event.target === cy && state.selected) {
            clearSelection();
        }
    });
    cy.on("tap", "edge", (event) => {
        setStatus(event.target.data("label").replace(/\n/g, "; "));
    });
    cy.on("dragfree", "node", (event) => {
        saveDrag(event.target);
        updateMinZoom();
    });
    cy.on("resize", updateMinZoom);
    cy.on("mouseover", "node", (event) => {
        if (event.target.id() !== CLUSTER_ID) {
            previewNeighbours(cy, event.target.id());
            elements.graph.title = tableSummary(event.target.id());
        }
    });
    cy.on("mouseout", "node", () => {
        clearPreview(cy);
        elements.graph.title = "";
    });
    cy.on("tap", hideContextMenu);
    cy.on("viewport", hideContextMenu);
    cy.on("cxttap", "node", (event) => {
        const id = event.target.id();
        if (id === CLUSTER_ID) {
            return;
        }
        if (state.selected !== id) {
            selectTable(id);
        }
        showContextMenu(event, id);
    });

    //-------------------------------------------------------------------//
    // Explore view: a table in the middle, the tables referencing it in the column to the left and the
    // tables it references in the column to the right. Exploring a shown table adds its own neighbours
    // the same way, one column further out.
    //-------------------------------------------------------------------//

    function newExploration(centre) {
        return { centre, columns: new Map(), column: new Map(), positions: {}, edges: new Set() };
    }

    function respaceColumn(exploration, columnIndex) {
        const ids = exploration.columns.get(columnIndex);
        ids.forEach((id, index) => {
            exploration.positions[id] = { x: columnIndex * EXPLORE_COLUMN_GAP, y: (index - (ids.length - 1) / 2) * EXPLORE_ROW_GAP };
        });
    }

    // Insert tables into a column next to the table that brought them in, then space the column evenly.
    function placeInColumn(exploration, ids, columnIndex, parentY) {
        if (ids.length === 0) {
            return;
        }
        if (!exploration.columns.has(columnIndex)) {
            exploration.columns.set(columnIndex, []);
        }
        const column = exploration.columns.get(columnIndex);
        const at = column.filter((id) => exploration.positions[id].y < parentY).length;
        column.splice(at, 0, ...ids);
        for (const id of ids) {
            exploration.column.set(id, columnIndex);
        }
        respaceColumn(exploration, columnIndex);
    }

    /**
     * Add a shown table's neighbours to the exploration: tables it references to its right, tables referencing it to its left.
     *
     * Tables already shown keep their place; only the edge to them is added. Each new table also gets its edges to every table shown.
     *
     * @param {Object} exploration The exploration to grow.
     * @param {string} id The table to explore.
     * @returns {string[]} The tables added.
     */
    function expandExploration(exploration, id) {
        const columnIndex = exploration.column.get(id);
        const parentY = exploration.positions[id].y;
        const right = [];
        const left = [];
        for (const neighbour of state.index.neighbours(id, "both")) {
            if (neighbour === id) {
                continue;
            }
            exploration.edges.add(pairKey(id, neighbour));
            if (!exploration.column.has(neighbour)) {
                (relation(id, neighbour) === "in" ? left : right).push(neighbour);
            }
        }
        right.sort();
        left.sort();
        placeInColumn(exploration, right, columnIndex + 1, parentY);
        placeInColumn(exploration, left, columnIndex - 1, parentY);

        const added = [...right, ...left];
        for (const newId of added) {
            for (const linked of state.index.neighbours(newId, "both")) {
                if (linked !== newId && exploration.column.has(linked)) {
                    exploration.edges.add(pairKey(newId, linked));
                }
            }
        }
        return added;
    }

    // Add the exploration's new tables (starting from the explored one, so they fan out of it) and edges, and move everything into its column.
    function renderExploration(fromId) {
        const exploration = state.exploration;
        const origin = exploreCy.getElementById(fromId);
        const start = origin.nonempty() ? { ...origin.position() } : exploration.positions[fromId];
        exploreCy.batch(() => {
            for (const id of exploration.column.keys()) {
                if (exploreCy.getElementById(id).empty()) {
                    exploreCy.add(starItem(id, start, ["labelled"]));
                }
            }
            for (const key of exploration.edges) {
                if (exploreCy.getElementById(key).empty() && state.pairsByKey.has(key)) {
                    exploreCy.add(pairEdgeItem(state.pairsByKey.get(key), exploration.positions));
                }
            }
            for (const edge of exploreCy.edges()) {
                edge.data("curve", edgeCurve(exploration.positions[edge.source().id()], exploration.positions[edge.target().id()]));
            }
        });
        for (const node of exploreCy.nodes()) {
            node.animate({ position: exploration.positions[node.id()] }, { duration: 350 });
        }
        setTimeout(() => fitTo(exploreCy.nodes(), { maxZoom: MAX_FOCUS_ZOOM, animate: true }), 380);
    }

    function exploreStatus() {
        const exploration = state.exploration;
        return `Explorer: ${exploration.column.size} tables around ${exploration.centre}. Right-click a table to explore it; click Explorer to go back to Galaxy.`;
    }

    function startExploration(id) {
        state.exploration = newExploration(id);
        placeInColumn(state.exploration, [id], 0, 0);
        expandExploration(state.exploration, id);
        exploreCy.elements().remove();
        showExploreView();
        renderExploration(id);
        selectInExplore(id);
    }

    function exploreFurther(id) {
        const added = expandExploration(state.exploration, id);
        renderExploration(id);
        selectInExplore(id);
        if (added.length === 0) {
            setStatus(`${id} has no tables to add that aren't shown already. ${exploreStatus()}`);
        }
    }

    function selectInExplore(id) {
        state.selected = id;
        state.path = null;
        exploreCy.batch(() => {
            exploreCy.elements().removeClass(EXPLORE_HIGHLIGHT_CLASSES);
            colourNeighbours(exploreCy, id, "");
        });
        showDetails(id);
        writeHash();
        setStatus(exploreStatus());
    }

    function showExploreView() {
        state.view = "explore";
        elements.title.textContent = "Explorer";
        hideContextMenu();
        elements.graph.hidden = true;
        elements.exploreGraph.hidden = false;
        elements.spreadControl.hidden = true;
        exploreCy.resize();
    }

    function showGalaxyView() {
        if (state.view === "galaxy") {
            return;
        }
        state.view = "galaxy";
        elements.title.textContent = "Galaxy";
        hideContextMenu();
        elements.exploreGraph.hidden = true;
        elements.graph.hidden = false;
        elements.spreadControl.hidden = false;
        cy.resize();
        if (state.selected) {
            selectTable(state.selected);
        } else {
            setStatus(state.summary);
        }
    }

    // The title shows the current view's name and switches between Galaxy and the last exploration in Explorer.
    function toggleView() {
        if (!state.index) {
            return;
        }
        if (state.view === "explore") {
            showGalaxyView();
        } else if (state.exploration) {
            showExploreView();
            if (state.selected && exploreCy.getElementById(state.selected).nonempty()) {
                selectInExplore(state.selected);
            } else {
                clearSelection();
            }
        } else {
            setStatus("Right-click a table and pick Explore to open it in Explorer.");
        }
    }

    /**
     * Select a table in the current view, falling back to Galaxy when Explorer doesn't show it.
     *
     * @param {string} id The table id.
     * @param {{zoom?: boolean}} [options] `zoom` zooms to the table and its neighbours.
     */
    function openTable(id, options = {}) {
        const node = exploreCy.getElementById(id);
        if (state.view === "explore" && node.nonempty()) {
            selectInExplore(id);
            if (options.zoom) {
                zoomTo(node.closedNeighborhood("node"));
            }
            return;
        }
        showGalaxyView();
        selectTable(id, options);
    }

    exploreCy.on("tap", "node", (event) => selectInExplore(event.target.id()));
    exploreCy.on("dbltap", "node", (event) => openTable(event.target.id(), { zoom: true }));
    exploreCy.on("tap", (event) => {
        hideContextMenu();
        if (event.target === exploreCy && state.selected) {
            clearSelection();
        }
    });
    exploreCy.on("tap", "edge", (event) => {
        setStatus(event.target.data("label").replace(/\n/g, "; "));
    });
    exploreCy.on("viewport", hideContextMenu);
    exploreCy.on("cxttap", "node", (event) => {
        if (state.selected !== event.target.id()) {
            selectInExplore(event.target.id());
        }
        showContextMenu(event, event.target.id());
    });
    exploreCy.on("mouseover", "node", (event) => {
        previewNeighbours(exploreCy, event.target.id());
        elements.exploreGraph.title = tableSummary(event.target.id());
    });
    exploreCy.on("mouseout", "node", () => {
        clearPreview(exploreCy);
        elements.exploreGraph.title = "";
    });

    //-------------------------------------------------------------------//
    // Right-click menu
    //-------------------------------------------------------------------//

    function showContextMenu(event, id) {
        const pane = elements.graph.parentElement.getBoundingClientRect();
        state.menuTarget = id;
        elements.contextMenu.style.left = `${event.originalEvent.clientX - pane.left}px`;
        elements.contextMenu.style.top = `${event.originalEvent.clientY - pane.top}px`;
        elements.contextMenu.hidden = false;
    }

    function hideContextMenu() {
        elements.contextMenu.hidden = true;
    }

    elements.contextExplore.addEventListener("click", () => {
        const id = state.menuTarget;
        hideContextMenu();
        if (state.view === "explore") {
            exploreFurther(id);
        } else {
            startExploration(id);
        }
    });
    for (const container of [elements.graph, elements.exploreGraph]) {
        container.addEventListener("contextmenu", (event) => event.preventDefault());
    }
    document.addEventListener("mousedown", (event) => {
        if (!elements.contextMenu.hidden && !elements.contextMenu.contains(event.target)) {
            hideContextMenu();
        }
    });
    document.addEventListener("keydown", (event) => {
        if (event.key === "Escape") {
            hideContextMenu();
        }
    });
    elements.title.addEventListener("click", toggleView);

    //-------------------------------------------------------------------//
    // Details panel (only shown while a table is selected)
    //-------------------------------------------------------------------//

    function element(tag, options = {}, children = []) {
        const node = document.createElement(tag);
        if (options.text !== undefined) {
            node.textContent = options.text;
        }
        if (options.className) {
            node.className = options.className;
        }
        if (options.title) {
            node.title = options.title;
        }
        for (const child of children) {
            if (child) {
                node.append(child);
            }
        }
        return node;
    }

    // The schema has no table-level comments, so table tooltips summarise the table instead.
    function tableSummary(id) {
        const node = state.index.nodes.get(id);
        if (!node) {
            return id;
        }
        const linked = `${state.index.degree(id)} linked tables`;
        if (node.missing) {
            return `${node.table}\nNot in the schema, referenced by other tables\n${linked}`;
        }
        const keys = node.keys.length > 0 ? `key: ${node.keys.join(", ")}` : "no key columns";
        return `${node.table}\n${node.fields} fields, ${keys}\nversions ${node.versions.join(", ")}, ${linked}`;
    }

    function tableLink(id, text = id) {
        const button = element("button", { text, className: "link", title: tableSummary(id) });
        button.type = "button";
        button.addEventListener("click", () => openTable(id, { zoom: true }));
        return button;
    }

    function actionButton(text, onClick, title) {
        const button = element("button", { text, title, className: "bubble" });
        button.type = "button";
        button.addEventListener("click", onClick);
        return button;
    }

    function setDetailsVisible(visible) {
        elements.details.hidden = !visible;
    }

    function hideDetails() {
        setDetailsVisible(false);
        elements.details.replaceChildren();
    }

    async function showDetails(id) {
        setDetailsVisible(true);
        let details = state.detailsCache.get(id);
        if (!details) {
            elements.details.replaceChildren(element("p", { className: "muted", text: `Loading ${id}...` }));
            try {
                details = await state.source.tableDetails(state.game, `${id}_tables`);
                state.detailsCache.set(id, details);
            } catch (error) {
                elements.details.replaceChildren(element("p", { className: "muted", text: `Couldn't load ${id}: ${error.message}` }));
                return;
            }
        }

        // The user may have clicked another table while this one was loading.
        if (state.selected === id || (state.path && state.path[0] === id)) {
            renderDetails(id, details);
        }
    }

    function renderDetails(id, details) {
        const node = state.index.nodes.get(id);
        const degree = state.index.degree(id);
        const newest = details.definitions[0];

        const header = element("div", { className: "details-header" }, [
            element("h2", { text: details.table, title: tableSummary(id) }),
            actionButton("Close", clearSelection, "Deselect this table"),
        ]);

        const fieldsBadge = newest ? element("span", { className: "badge", text: `${newest.fields.length} fields` }) : null;
        const definitions = details.missing ? null : definitionSections(details, fieldsBadge);
        const badges = element("div", { className: "badges" }, [
            details.missing ? element("span", { className: "badge badge-warning", text: "Not in schema" }) : null,
            newest ? versionBadge(details.definitions, definitions.showVersion) : null,
            details.definitions.length > 1 ? element("span", { className: "badge", text: `${details.definitions.length} versions` }) : null,
            fieldsBadge,
            node?.localised ? element("span", { className: "badge", text: "Localised" }) : null,
            element("span", { className: "badge", text: `${degree} linked tables` }),
        ]);

        const actions = element("div", { className: "actions" }, [
            actionButton("Zoom to", () => openTable(id, { zoom: true }), "Zoom to this table and its neighbours"),
            actionButton("Copy name", () => copyText(details.table), "Copy the full table name"),
        ]);

        const sections = [header, badges, actions];

        if (details.missing) {
            sections.push(element("p", {
                className: "muted",
                text: "Other tables reference this table, but the schema has no definition for it.",
            }));
        } else {
            sections.push(...definitions.sections);
        }

        sections.push(referencedBySection(details));
        elements.details.replaceChildren(...sections);
    }

    // A section that can be folded away. Whether it's open is remembered for the next table.
    function collapsibleSection(key, title, children) {
        const section = element("details", { className: "section" }, [element("summary", { text: title }), element("div", { className: "section-body" }, children)]);
        section.open = state.openSections[key];
        section.addEventListener("toggle", () => {
            state.openSections[key] = section.open;
        });
        return section;
    }

    // The version badge. With several versions it's a picker that switches the Fields and Localised fields sections.
    function versionBadge(definitions, showVersion) {
        if (definitions.length === 1) {
            return element("span", { className: "badge", text: `v${definitions[0].version}` });
        }
        const select = element("select", { className: "badge version-badge", title: "Definition version: pick one to see its fields" });
        for (const definition of definitions) {
            const option = element("option", { text: `v${definition.version}` });
            option.value = String(definition.version);
            select.append(option);
        }
        select.addEventListener("change", () => showVersion(definitions.find((definition) => String(definition.version) === select.value)));
        return select;
    }

    /**
     * Build the Fields section, plus a Localised fields section if any version has some.
     *
     * @param {Object} details The table details.
     * @param {HTMLElement} fieldsBadge The badge showing the field count, kept in step with the shown version.
     * @returns {{sections: HTMLElement[], showVersion: Function}} The sections, and a function showing another version in them.
     */
    function definitionSections(details, fieldsBadge) {
        const fieldsBody = element("div");
        const localisedBody = element("div");
        const fields = collapsibleSection("fields", "", [fieldsBody]);
        const localised = collapsibleSection("localised", "", [localisedBody]);
        const showVersion = (definition) => {
            fields.querySelector("summary").textContent = `Fields (${definition.fields.length})`;
            fieldsBody.replaceChildren(fieldsTable(definition));
            localised.querySelector("summary").textContent = `Localised fields (${definition.localised_fields.length})`;
            localisedBody.replaceChildren(localisedList(definition));
            fieldsBadge.textContent = `${definition.fields.length} fields`;
        };
        showVersion(details.definitions[0]);

        const hasLocalised = details.definitions.some((definition) => definition.localised_fields.length > 0);
        return { sections: hasLocalised ? [fields, localised] : [fields], showVersion };
    }

    function fieldsTable(definition) {
        const header = element("tr", {}, [
            element("th", { text: "Name", className: "col-name" }),
            element("th", { text: "Type", className: "col-type" }),
            element("th", { text: "Details" }),
        ]);
        const rows = definition.fields.map((field) => fieldRow(field));
        return element("table", { className: "fields" }, [element("thead", {}, [header]), element("tbody", {}, rows)]);
    }

    function localisedList(definition) {
        if (definition.localised_fields.length === 0) {
            return element("p", { className: "muted", text: "None in this version." });
        }
        return element("ul", { className: "reference-list" }, definition.localised_fields.map((field) => element("li", { text: field.name })));
    }

    function fieldRow(field) {
        const { explanation, ...otherNotes } = field.patch_notes || {};
        const comment = [field.description, explanation].filter(Boolean).join("\n");
        const name = element("td", { className: field.is_key ? "key" : "" });
        name.append(element("span", { text: field.name, className: comment ? "field-name has-comment" : "field-name" }));

        const details = element("td");
        if (field.is_reference) {
            const [targetId, targetColumn] = field.is_reference;
            details.append(element("span", { text: "-> " }), tableLink(targetId), element("span", { text: `.${targetColumn}` }));
            if (Array.isArray(field.lookup) && field.lookup.length > 0) {
                details.append(element("span", { className: "note", text: `lookup: ${field.lookup.join(", ")}` }));
            }
        }

        const noteLines = [];
        if (field.default_value !== null && field.default_value !== "") {
            noteLines.push(`default: ${field.default_value}`);
        }
        if (field.is_filename) {
            noteLines.push(field.filename_relative_path ? `file: ${field.filename_relative_path.join("; ")}` : "file path");
        }
        if (field.is_bitwise > 1) {
            noteLines.push(`bitwise (${field.is_bitwise} flags)`);
        }
        const enumValues = Object.entries(field.enum_values || {});
        if (enumValues.length > 0) {
            noteLines.push(`enum: ${enumValues.map(([key, value]) => `${key}=${value}`).join(", ")}`);
        }
        if (field.is_part_of_colour !== null) {
            noteLines.push(`colour part ${field.is_part_of_colour}`);
        }
        for (const [key, value] of Object.entries(otherNotes)) {
            noteLines.push(`${key}: ${value}`);
        }
        for (const line of noteLines) {
            details.append(element("span", { className: "note", text: line }));
        }

        const row = element("tr", {}, [name, element("td", { text: field.field_type_name || String(field.field_type) }), details]);
        row.title = comment;
        return row;
    }

    function referencedBySection(details) {
        const columns = Object.entries(details.referenced_by || {});
        const tableCount = new Set(columns.flatMap(([, tables]) => Object.keys(tables))).size;
        const children = [];
        if (columns.length === 0) {
            children.push(element("p", { className: "muted", text: "No table references this one." }));
        }
        for (const [column, tables] of columns) {
            children.push(element("p", { text: `${column} (${Object.keys(tables).length} tables)` }));
            const items = Object.entries(tables).map(([table, sourceColumns]) => {
                const id = table.replace(/_tables$/, "");
                return element("li", {}, [tableLink(id), element("span", { className: "muted", text: ` .${sourceColumns.join(", .")}` })]);
            });
            children.push(element("ul", { className: "reference-list" }, items));
        }
        return collapsibleSection("referencedBy", `Referenced by (${tableCount})`, children);
    }

    //-------------------------------------------------------------------//
    // Path finder (floating at the bottom left)
    //-------------------------------------------------------------------//

    function resetPathResult() {
        elements.pathResult.replaceChildren();
        elements.pathClear.hidden = true;
    }

    function findPath() {
        const fromId = elements.pathFrom.value.trim().replace(/_tables$/, "");
        const toId = elements.pathTo.value.trim().replace(/_tables$/, "");
        const unknown = [fromId, toId].find((id) => !state.index.nodes.has(id));
        if (unknown !== undefined) {
            elements.pathResult.replaceChildren(element("p", { className: "note", text: unknown ? `Unknown table: ${unknown}` : "Pick a table to start from and one to reach." }));
            return;
        }
        const path = state.index.shortestPath(fromId, toId);
        if (!path) {
            elements.pathResult.replaceChildren(element("p", { className: "note", text: `No path from ${fromId} to ${toId}.` }));
            return;
        }

        showGalaxyView();
        state.selected = path[0];
        state.path = path;
        highlightPath(path);
        showDetails(path[0]);
        writeHash();
        elements.pathResult.replaceChildren(pathSteps(path));
        elements.pathClear.hidden = false;
        setStatus(`Path of ${path.length - 1} steps from ${path[0]} to ${path[path.length - 1]}.`);
    }

    function clearPath() {
        resetPathResult();
        if (state.selected) {
            openTable(state.selected);
        } else {
            clearSelection();
        }
    }

    elements.pathBox.addEventListener("submit", (event) => {
        event.preventDefault();
        findPath();
    });
    elements.pathClear.addEventListener("click", clearPath);

    function pathSteps(path) {
        const items = path.map((id, index) => {
            const item = element("li", {}, [tableLink(id)]);
            if (index + 1 < path.length) {
                const columns = state.index.edgesBetween(id, path[index + 1])
                    .map((edge) => `${edge.from}.${edge.from_column} -> ${edge.to}.${edge.to_column}`);
                item.append(element("span", { className: "note", text: columns.join("; ") }));
            }
            return item;
        });
        return element("ol", { className: "path-steps" }, items);
    }

    async function copyText(text) {
        try {
            await navigator.clipboard.writeText(text);
            setStatus(`Copied ${text}`);
        } catch (error) {
            setStatus(`Couldn't copy: ${error.message}`, true);
        }
    }

    //-------------------------------------------------------------------//
    // Search
    //-------------------------------------------------------------------//

    function highlightSearchMatches(nodes) {
        cy.batch(() => {
            cy.$(".search-match").removeClass("search-match");
            for (const node of nodes) {
                cy.getElementById(node.id).addClass("search-match");
            }
        });
    }

    // Blue search highlights stay until the search box is emptied (or cleared with Escape).
    function clearSearch() {
        elements.search.value = "";
        highlightSearchMatches([]);
    }

    /**
     * Show a purple list of matching tables under (or above) a text box, picked with the mouse or arrow keys and Enter.
     *
     * @param {HTMLInputElement} input The text box.
     * @param {HTMLElement} list The list element to fill with suggestions.
     * @param {{onMatches?: Function, onChoose: Function, onEscape?: Function}} handlers Called with the matches as they
     *   change, with the picked table, and when Escape is pressed.
     */
    function attachTableSuggestions(input, list, { onMatches = () => {}, onChoose, onEscape = () => {} }) {
        let matches = [];
        let active = -1;

        const render = () => {
            const items = matches.map((node, index) => {
                const hint = node.missing ? "not in schema" : `${state.index.degree(node.id)} linked`;
                const item = element("li", {}, [element("span", { text: node.id }), element("span", { className: "hint", text: hint })]);
                item.setAttribute("role", "option");
                item.setAttribute("aria-selected", String(index === active));
                item.addEventListener("mousedown", (event) => {
                    event.preventDefault();
                    choose(index);
                });
                return item;
            });
            list.replaceChildren(...items);
            list.hidden = items.length === 0;
        };
        const update = () => {
            matches = state.index ? state.index.search(input.value, SEARCH_LIMIT) : [];
            active = matches.length > 0 ? 0 : -1;
            render();
            onMatches(matches);
        };
        const choose = (index) => {
            const node = matches[index];
            if (node) {
                list.hidden = true;
                onChoose(node);
            }
        };

        input.addEventListener("input", update);
        input.addEventListener("focus", update);
        input.addEventListener("blur", () => {
            list.hidden = true;
        });
        input.addEventListener("keydown", (event) => {
            if ((event.key === "ArrowDown" || event.key === "ArrowUp") && matches.length > 0) {
                event.preventDefault();
                active = (active + (event.key === "ArrowDown" ? 1 : -1) + matches.length) % matches.length;
                render();
                list.children[active]?.scrollIntoView({ block: "nearest" });
            } else if (event.key === "Enter" && !list.hidden) {
                event.preventDefault();
                choose(Math.max(active, 0));
            } else if (event.key === "Escape") {
                event.preventDefault();
                list.hidden = true;
                onEscape();
            }
        });
    }

    attachTableSuggestions(elements.search, elements.searchResults, {
        onMatches: highlightSearchMatches,
        onChoose: (node) => openTable(node.id, { zoom: true }),
        onEscape: clearSearch,
    });
    attachTableSuggestions(elements.pathFrom, elements.pathFromSuggestions, {
        onChoose: (node) => {
            elements.pathFrom.value = node.id;
        },
    });
    attachTableSuggestions(elements.pathTo, elements.pathToSuggestions, {
        onChoose: (node) => {
            elements.pathTo.value = node.id;
        },
    });

    //-------------------------------------------------------------------//
    // URL fragment: #game=<game>&table=<id>, so views can be bookmarked.
    //-------------------------------------------------------------------//

    function readHash() {
        const params = new URLSearchParams(window.location.hash.slice(1));
        return { game: params.get("game"), table: params.get("table")?.replace(/_tables$/, "") || null };
    }

    function writeHash() {
        const params = new URLSearchParams();
        if (state.game) {
            params.set("game", state.game);
        }
        if (state.selected) {
            params.set("table", state.selected);
        }
        const hash = params.toString();
        if (window.location.hash.slice(1) !== hash) {
            window.history.replaceState(null, "", hash ? `#${hash}` : window.location.pathname + window.location.search);
        }
    }

    window.addEventListener("hashchange", () => {
        const { game, table } = readHash();
        if (game && state.source && game !== state.game) {
            loadFromServer(game);
        } else if (table && state.index?.nodes.has(table) && table !== state.selected) {
            openTable(table, { zoom: true });
        }
    });

    //-------------------------------------------------------------------//
    // Startup
    //-------------------------------------------------------------------//

    if (WsSource.isAvailable()) {
        loadFromServer(readHash().game);
    } else {
        setStatus("Open the explorer from RPFM: Tools > Schema Explorer.", true);
    }
})();
