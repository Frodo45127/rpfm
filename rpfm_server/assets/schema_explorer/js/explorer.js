// Schema explorer UI: wires the data source, the graph view (Cytoscape.js) and the details panel together.
//
// The whole schema is drawn at once on a saved ForceAtlas2 layout (see layout.js). Selecting a table
// recolours it and its neighbours instead of redrawing, so the map never moves around. Dragged stars
// are saved into the layout.
// Everything shown comes from the schema (definitions and references). No table data is ever read.
// All schema strings go into the DOM through `textContent`, never `innerHTML`, as they can contain anything.

(() => {
    "use strict";

    const { ApiSource, FileSource } = window.RpfmDataSource;
    const { GraphIndex, pairKey } = window.RpfmGraphIndex;
    const { layoutFor, saveMovedStars } = window.RpfmLayout;

    const SETTINGS_KEY = "rpfm-schema-explorer-settings";
    const SEARCH_LIMIT = 30;
    const CLUSTER_ID = "__tables_without_references__";
    const HUB_THRESHOLD = 40;
    const MAX_FOCUS_ZOOM = 1.6;
    const EDGE_CURVATURE = 0.12;
    const MAX_EDGE_CURVE = 90;
    const CLUSTER_MARGIN = 30;

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
        cluster: "#6b7bb5",
    };

    const elements = {
        gameSelect: document.getElementById("game-select"),
        fileInput: document.getElementById("file-input"),
        sourceLabel: document.getElementById("source-label"),
        search: document.getElementById("search"),
        searchResults: document.getElementById("search-results"),
        overview: document.getElementById("overview"),
        graph: document.getElementById("graph"),
        emptyState: document.getElementById("empty-state"),
        status: document.getElementById("graph-status"),
        details: document.getElementById("details"),
        dropOverlay: document.getElementById("drop-overlay"),
        tableNames: document.getElementById("table-names"),
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
        clusterCentre: null,
        lastGame: loadLastGame(),
        searchMatches: [],
        searchActive: -1,
    };

    const cy = window.cytoscape({
        container: elements.graph,
        style: graphStyle(),
        minZoom: 0.02,
        maxZoom: 5,
        boxSelectionEnabled: false,
    });

    //-------------------------------------------------------------------//
    // Remembered game (API mode)
    //-------------------------------------------------------------------//

    // localStorage can be missing or throw (private windows, file:// in some browsers), so this is best-effort.
    function loadLastGame() {
        try {
            return JSON.parse(window.localStorage.getItem(SETTINGS_KEY) || "{}").game || null;
        } catch (error) {
            return null;
        }
    }

    function saveLastGame(game) {
        try {
            window.localStorage.setItem(SETTINGS_KEY, JSON.stringify({ game }));
        } catch (error) {
            console.debug("Couldn't save the selected game:", error.message);
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
            { selector: nodes, style: { "opacity": opacity, "underlay-opacity": 0, "min-zoomed-font-size": 0, "text-opacity": 1 } },
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
                    "label": "data(id)",
                    "color": COLOURS.label,
                    "font-size": 9,
                    "min-zoomed-font-size": 14,
                    "text-valign": "bottom",
                    "text-margin-y": 2,
                    "text-outline-color": COLOURS.labelOutline,
                    "text-outline-width": 2,
                    "text-opacity": 0.85,
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
            { selector: "node.labelled", style: { "min-zoomed-font-size": 0, "text-opacity": 1 } },
            { selector: "edge:selected", style: { "label": "data(label)", "opacity": 1 } },
        ];
    }

    //-------------------------------------------------------------------//
    // Loading data
    //-------------------------------------------------------------------//

    async function useSource(source) {
        state.source = source;
        state.detailsCache.clear();
        const games = await source.games();

        if (source.kind === "api") {
            elements.gameSelect.replaceChildren(...games.map((game) => {
                const option = document.createElement("option");
                option.value = game.key;
                option.textContent = game.schema_available ? game.display_name : `${game.display_name} (no schema)`;
                option.disabled = !game.schema_available;
                return option;
            }));
            elements.gameSelect.hidden = false;
            elements.sourceLabel.textContent = "Connected to rpfm_server";
        } else {
            elements.gameSelect.hidden = true;
            elements.sourceLabel.textContent = `Loaded: ${source.fileName}`;
        }

        const hashGame = readHash().game;
        const available = games.filter((game) => game.schema_available);
        const preferred = available.find((game) => game.key === hashGame)
            || available.find((game) => game.key === state.lastGame)
            || available[0];
        if (!preferred) {
            setStatus("No game has a schema available.", true);
            return;
        }
        elements.gameSelect.value = preferred.key;
        await loadGame(preferred.key);
    }

    async function loadGame(game) {
        setStatus("Loading schema graph...");
        hideDetails();
        let layout;
        try {
            const graph = await state.source.graph(game);
            state.game = game;
            state.sourceKey = state.source.kind === "api" ? `api:${game}` : `file:${game}`;
            state.graph = graph;
            state.index = new GraphIndex(graph);
            state.detailsCache.clear();
            if (state.source.kind === "api") {
                state.lastGame = game;
                saveLastGame(game);
            }
            layout = await layoutFor(state.sourceKey, graph, ({ iterations, movement }) => {
                setStatus(`Laying out the schema until it settles: ${iterations} iterations, movement ${movement.toFixed(4)}...`);
            });
        } catch (error) {
            setStatus(`Couldn't load the schema graph: ${error.message}`, true);
            return;
        }

        elements.emptyState.hidden = true;
        elements.search.disabled = false;
        elements.overview.disabled = false;
        elements.tableNames.replaceChildren(...[...state.index.nodes.keys()].map((id) => {
            const option = document.createElement("option");
            option.value = id;
            return option;
        }));

        renderOverview(layout);
        const tableCount = state.graph.nodes.filter((node) => !node.missing).length;
        const layoutNote = {
            cached: "saved layout",
            updated: `layout updated for schema changes in ${layout.iterations} iterations`,
            new: `new layout, settled in ${layout.iterations} iterations`,
        }[layout.status];
        setStatus(`${tableCount} tables, ${state.graph.edges.length} references, ${layout.isolated.length} without references (${layoutNote}). Search or click a star.`);

        const hashTable = readHash().table;
        if (hashTable && state.index.nodes.has(hashTable)) {
            selectTable(hashTable, { zoom: true });
        }
    }

    async function loadFile(file) {
        setStatus(`Reading ${file.name}...`);
        try {
            const text = await file.text();
            setStatus(`Parsing ${file.name}...`);

            // Yield once so the status paints before the (synchronous) parse of a multi-MB file.
            await new Promise((resolve) => setTimeout(resolve, 0));
            const source = FileSource.fromText(file.name, text);
            await useSource(source);
        } catch (error) {
            setStatus(`Couldn't load ${file.name}: ${error.message}`, true);
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

    function renderOverview(layout) {
        const isolated = new Set(layout.isolated);
        state.isolated = isolated;
        state.clusterCentre = layout.clusterCentre ? { ...layout.clusterCentre } : null;
        const items = [];
        if (isolated.size > 0) {
            const { centre, radius } = boundingCircle(layout.isolated.map((id) => layout.positions[id]), CLUSTER_MARGIN);
            items.push({
                group: "nodes",
                data: { id: CLUSTER_ID, label: `Tables without references (${isolated.size})`, diameter: radius * 2 },
                position: centre,
                selectable: false,
            });
        }

        for (const node of state.graph.nodes) {
            const classes = [];
            if (node.missing) {
                classes.push("missing");
            }
            if (state.index.degree(node.id) > HUB_THRESHOLD) {
                classes.push("hub");
            }
            items.push({
                group: "nodes",
                data: { id: node.id, degree: state.index.degree(node.id) },
                position: { ...layout.positions[node.id] },
                classes: classes.join(" "),
            });
        }

        // One edge per pair of tables: pointing the way the references go, or both ways when they go both ways.
        for (const pair of state.index.linkedPairs()) {
            const hasForward = pair.forward.length > 0;
            const hasBackward = pair.backward.length > 0;
            const [source, target] = hasForward ? [pair.a, pair.b] : [pair.b, pair.a];
            const classes = [];
            if (pair.a === pair.b) {
                classes.push("loop");
            } else if (hasForward && hasBackward) {
                classes.push("bidirectional");
            }
            items.push({
                group: "edges",
                data: {
                    id: pair.key,
                    source,
                    target,
                    label: pairLabel(pair),
                    curve: pair.a === pair.b ? 0 : edgeCurve(layout.positions[source], layout.positions[target]),
                },
                classes: classes.join(" "),
            });
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

    function showOverview() {
        state.selected = null;
        state.path = null;
        clearHighlight();
        cy.$(":selected").unselect();
        hideDetails();
        fitTo(cy.nodes(), { maxZoom: Infinity, animate: true });
        writeHash();
        setStatus("Showing the whole schema.");
    }

    // Fit the view to some nodes from their positions alone. Cytoscape's own fit measures every label,
    // which takes seconds on a whole schema.
    function fitTo(nodes, { maxZoom, animate }) {
        if (nodes.length === 0) {
            return;
        }
        const extent = nodes.boundingBox({ includeLabels: false, includeOverlays: false, includeUnderlays: false });
        const padding = 50;
        const fitZoom = Math.min(
            maxZoom,
            cy.maxZoom(),
            (cy.width() - 2 * padding) / Math.max(extent.w, 1),
            (cy.height() - 2 * padding) / Math.max(extent.h, 1),
        );
        const zoom = Math.max(fitZoom, cy.minZoom());
        const pan = { x: cy.width() / 2 - zoom * (extent.x1 + extent.w / 2), y: cy.height() / 2 - zoom * (extent.y1 + extent.h / 2) };
        cy.stop();
        if (animate) {
            cy.animate({ zoom, pan }, { duration: 450 });
        } else {
            cy.viewport({ zoom, pan });
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
    function colourNeighbours(id, prefix) {
        const kinds = state.index.neighbours(id, "both").filter((neighbour) => neighbour !== id).map((neighbour) => [neighbour, relation(id, neighbour)]);
        cy.getElementById(id).addClass(`${prefix}selected`);
        for (const [neighbour, kind] of kinds) {
            cy.getElementById(neighbour).addClass(`${prefix}${kind}`);
            cy.getElementById(pairKey(id, neighbour)).addClass(`${prefix}${kind}`);
        }
        cy.getElementById(pairKey(id, id)).addClass(`${prefix}out`);
        return kinds;
    }

    function highlightNeighbours(id) {
        const counts = { in: 0, out: 0, both: 0 };
        let kinds = [];
        cy.batch(() => {
            clearHighlight();
            kinds = colourNeighbours(id, "");
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

    function clearPreview() {
        cy.batch(() => cy.$(".hover-selected, .hover-in, .hover-out, .hover-both").removeClass(PREVIEW_CLASSES));
    }

    function previewNeighbours(id) {
        cy.batch(() => {
            cy.$(".hover-selected, .hover-in, .hover-out, .hover-both").removeClass(PREVIEW_CLASSES);
            colourNeighbours(id, "hover-");
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
            moves[star.id()] = { ...star.position() };
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
            state.clusterCentre = { x: state.clusterCentre.x + delta.x, y: state.clusterCentre.y + delta.y };
        }
        clusterDragFrom = { ...position };
    });

    cy.on("tap", "node", (event) => {
        if (event.target.id() !== CLUSTER_ID) {
            selectTable(event.target.id());
        } else if (state.selected) {
            showOverview();
        }
    });
    cy.on("dbltap", "node", (event) => {
        if (event.target.id() !== CLUSTER_ID) {
            selectTable(event.target.id(), { zoom: true });
        }
    });
    cy.on("tap", (event) => {
        if (event.target === cy && state.selected) {
            showOverview();
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
            previewNeighbours(event.target.id());
        }
    });
    cy.on("mouseout", "node", clearPreview);

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

    function tableLink(id, text = id) {
        const button = element("button", { text, className: "link", title: "Select this table" });
        button.type = "button";
        button.addEventListener("click", () => selectTable(id, { zoom: true }));
        return button;
    }

    function actionButton(text, onClick, title) {
        const button = element("button", { text, title });
        button.type = "button";
        button.addEventListener("click", onClick);
        return button;
    }

    // The graph area changes size with the panel, so Cytoscape has to be told.
    function setDetailsVisible(visible) {
        if (elements.details.hidden === !visible) {
            return;
        }
        elements.details.hidden = !visible;
        cy.resize();
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
            element("h2", { text: details.table }),
            actionButton("Close", showOverview, "Deselect and go back to the whole schema"),
        ]);

        const badges = element("div", { className: "badges" }, [
            details.missing ? element("span", { className: "badge badge-warning", text: "Not in schema" }) : null,
            newest ? element("span", { className: "badge", text: `v${newest.version}` }) : null,
            details.definitions.length > 1 ? element("span", { className: "badge", text: `${details.definitions.length} versions` }) : null,
            newest ? element("span", { className: "badge", text: `${newest.fields.length} fields` }) : null,
            node?.localised ? element("span", { className: "badge", text: "Localised" }) : null,
            element("span", { className: "badge", text: `${degree} linked tables` }),
        ]);

        const actions = element("div", { className: "actions" }, [
            actionButton("Zoom to", () => selectTable(id, { zoom: true }), "Zoom to this table and its neighbours"),
            actionButton("Copy name", () => copyText(details.table), "Copy the full table name"),
        ]);

        const sections = [header, badges, actions];

        if (details.missing) {
            sections.push(element("p", {
                className: "muted",
                text: "Other tables reference this table, but the schema has no definition for it.",
            }));
        } else {
            sections.push(definitionSection(details));
        }

        sections.push(referencedBySection(details));
        sections.push(pathSection(id));
        elements.details.replaceChildren(...sections);
    }

    function definitionSection(details) {
        const container = element("div");
        const heading = element("h3", { text: "Fields" });
        const body = element("div");
        container.append(heading);

        if (details.definitions.length > 1) {
            const select = element("select", { title: "Definition version" });
            for (const definition of details.definitions) {
                const option = element("option", { text: `Version ${definition.version}` });
                option.value = String(definition.version);
                select.append(option);
            }
            select.addEventListener("change", () => {
                const definition = details.definitions.find((d) => String(d.version) === select.value);
                body.replaceChildren(fieldsTable(definition));
            });
            heading.append(" ", select);
        }

        body.append(fieldsTable(details.definitions[0]));
        container.append(body);
        return container;
    }

    function fieldsTable(definition) {
        const container = element("div");
        const header = element("tr", {}, [
            element("th", { text: "Name", className: "col-name" }),
            element("th", { text: "Type", className: "col-type" }),
            element("th", { text: "Details" }),
        ]);
        const rows = definition.fields.map((field) => fieldRow(field));
        container.append(element("table", { className: "fields" }, [element("thead", {}, [header]), element("tbody", {}, rows)]));

        if (definition.localised_fields.length > 0) {
            container.append(element("h3", { text: "Localised fields" }));
            const list = element("ul", { className: "reference-list" }, definition.localised_fields.map((field) => element("li", { text: field.name })));
            container.append(list);
        }
        return container;
    }

    function fieldRow(field) {
        const name = element("td", { text: field.name, className: field.is_key ? "key" : "", title: field.is_key ? "Key column" : "" });
        if (field.is_key) {
            name.append(element("span", { className: "note", text: "key" }));
        }

        const details = element("td");
        if (field.is_reference) {
            const [targetId, targetColumn] = field.is_reference;
            details.append(element("span", { text: "-> " }), tableLink(targetId), element("span", { text: `.${targetColumn}` }));
            if (Array.isArray(field.lookup) && field.lookup.length > 0) {
                details.append(element("span", { className: "note", text: `lookup: ${field.lookup.join(", ")}` }));
            }
        }

        const noteLines = [];
        if (field.description) {
            noteLines.push(field.description);
        }
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
        for (const [key, value] of Object.entries(field.patch_notes || {})) {
            noteLines.push(`${key}: ${value}`);
        }
        for (const line of noteLines) {
            details.append(element("span", { className: "note", text: line }));
        }

        return element("tr", {}, [name, element("td", { text: field.field_type_name || String(field.field_type) }), details]);
    }

    function referencedBySection(details) {
        const container = element("div", {}, [element("h3", { text: "Referenced by" })]);
        const columns = Object.entries(details.referenced_by || {});
        if (columns.length === 0) {
            container.append(element("p", { className: "muted", text: "No table references this one." }));
            return container;
        }

        for (const [column, tables] of columns) {
            container.append(element("p", { text: `${column} (${Object.keys(tables).length} tables)` }));
            const items = Object.entries(tables).map(([table, sourceColumns]) => {
                const id = table.replace(/_tables$/, "");
                return element("li", {}, [tableLink(id), element("span", { className: "muted", text: ` .${sourceColumns.join(", .")}` })]);
            });
            container.append(element("ul", { className: "reference-list" }, items));
        }
        return container;
    }

    function pathSection(fromId) {
        const input = element("input", { title: "Target table" });
        input.setAttribute("list", "table-names");
        input.placeholder = "Path to table...";
        const result = element("div");
        const find = () => {
            const toId = input.value.trim().replace(/_tables$/, "");
            const path = state.index.shortestPath(fromId, toId);
            if (!path) {
                result.replaceChildren(element("p", { className: "muted", text: `No path from ${fromId} to ${toId}.` }));
                return;
            }
            state.path = path;
            highlightPath(path);
            result.replaceChildren(pathSteps(path));
            setStatus(`Path of ${path.length - 1} steps from ${path[0]} to ${path[path.length - 1]}.`);
        };
        const button = actionButton("Find path", find);
        input.addEventListener("keydown", (event) => {
            if (event.key === "Enter") {
                find();
            }
        });

        return element("div", {}, [element("h3", { text: "Path" }), element("div", { className: "path-form" }, [input, button]), result]);
    }

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

    function updateSearch() {
        state.searchMatches = state.index ? state.index.search(elements.search.value, SEARCH_LIMIT) : [];
        state.searchActive = state.searchMatches.length > 0 ? 0 : -1;
        renderSearchResults();
    }

    function renderSearchResults() {
        const items = state.searchMatches.map((node, index) => {
            const hint = node.missing ? "not in schema" : `${state.index.degree(node.id)} linked`;
            const item = element("li", {}, [element("span", { text: node.id }), element("span", { className: "hint", text: hint })]);
            item.setAttribute("role", "option");
            item.setAttribute("aria-selected", String(index === state.searchActive));
            item.addEventListener("mousedown", (event) => {
                event.preventDefault();
                chooseSearchResult(index);
            });
            return item;
        });
        elements.searchResults.replaceChildren(...items);
        elements.searchResults.hidden = items.length === 0;
    }

    function chooseSearchResult(index) {
        const node = state.searchMatches[index];
        if (!node) {
            return;
        }
        elements.search.value = node.id;
        elements.searchResults.hidden = true;
        selectTable(node.id, { zoom: true });
    }

    elements.search.addEventListener("input", updateSearch);
    elements.search.addEventListener("focus", updateSearch);
    elements.search.addEventListener("blur", () => {
        elements.searchResults.hidden = true;
    });
    elements.search.addEventListener("keydown", (event) => {
        if (event.key === "ArrowDown" || event.key === "ArrowUp") {
            event.preventDefault();
            const step = event.key === "ArrowDown" ? 1 : -1;
            const count = state.searchMatches.length;
            if (count > 0) {
                state.searchActive = (state.searchActive + step + count) % count;
                renderSearchResults();
                elements.searchResults.children[state.searchActive]?.scrollIntoView({ block: "nearest" });
            }
        } else if (event.key === "Enter") {
            event.preventDefault();
            chooseSearchResult(Math.max(state.searchActive, 0));
        } else if (event.key === "Escape") {
            elements.searchResults.hidden = true;
        }
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
        if (state.source?.kind === "api" && state.game) {
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
        if (game && state.source?.kind === "api" && game !== state.game) {
            elements.gameSelect.value = game;
            loadGame(game);
        } else if (table && state.index?.nodes.has(table) && table !== state.selected) {
            selectTable(table, { zoom: true });
        }
    });

    //-------------------------------------------------------------------//
    // Controls, file picking and drag and drop
    //-------------------------------------------------------------------//

    elements.overview.addEventListener("click", showOverview);
    elements.gameSelect.addEventListener("change", () => loadGame(elements.gameSelect.value));
    elements.fileInput.addEventListener("change", () => {
        const file = elements.fileInput.files?.[0];
        if (file) {
            loadFile(file);
        }
        elements.fileInput.value = "";
    });

    // Counts nested dragenter/dragleave pairs so the overlay doesn't flicker over child elements.
    let dragDepth = 0;
    window.addEventListener("dragenter", (event) => {
        if (event.dataTransfer?.types.includes("Files")) {
            dragDepth++;
            elements.dropOverlay.hidden = false;
        }
    });
    window.addEventListener("dragleave", () => {
        dragDepth = Math.max(dragDepth - 1, 0);
        if (dragDepth === 0) {
            elements.dropOverlay.hidden = true;
        }
    });
    window.addEventListener("dragover", (event) => event.preventDefault());
    window.addEventListener("drop", (event) => {
        event.preventDefault();
        dragDepth = 0;
        elements.dropOverlay.hidden = true;
        const file = event.dataTransfer?.files?.[0];
        if (file) {
            loadFile(file);
        }
    });

    //-------------------------------------------------------------------//
    // Startup
    //-------------------------------------------------------------------//

    ApiSource.detect().then((source) => {
        if (source) {
            useSource(source).catch((error) => setStatus(`Couldn't load games: ${error.message}`, true));
        } else {
            setStatus("Open or drop a schema file to start.");
        }
    });
})();
