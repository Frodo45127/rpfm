// Schema explorer UI: wires the data source, the graph view (Cytoscape.js) and the details panel together.
//
// Everything shown comes from the schema (definitions and references). No table data is ever read.
// All schema strings go into the DOM through `textContent`, never `innerHTML`, as they can contain anything.

(() => {
    "use strict";

    const { ApiSource, FileSource } = window.RpfmDataSource;
    const { GraphIndex } = window.RpfmGraphIndex;

    const SETTINGS_KEY = "rpfm-schema-explorer-settings";
    const MAX_VISIBLE_NODES = 300;
    const SEARCH_LIMIT = 30;
    const MAX_FIT_ZOOM = 1.2;
    const DEFAULT_SETTINGS = { depth: 1, direction: "both", hideHubs: false, hubThreshold: 40, game: null };

    const elements = {
        gameSelect: document.getElementById("game-select"),
        fileInput: document.getElementById("file-input"),
        sourceLabel: document.getElementById("source-label"),
        search: document.getElementById("search"),
        searchResults: document.getElementById("search-results"),
        depth: document.getElementById("depth"),
        direction: document.getElementById("direction"),
        hideHubs: document.getElementById("hide-hubs"),
        hubThreshold: document.getElementById("hub-threshold"),
        relayout: document.getElementById("relayout"),
        showAll: document.getElementById("show-all"),
        graph: document.getElementById("graph"),
        emptyState: document.getElementById("empty-state"),
        status: document.getElementById("graph-status"),
        details: document.getElementById("details"),
        dropOverlay: document.getElementById("drop-overlay"),
        tableNames: document.getElementById("table-names"),
    };

    const state = {
        source: null,
        game: null,
        index: null,
        focus: null,
        selected: null,
        visible: new Set(),
        path: null,
        detailsCache: new Map(),
        settings: loadSettings(),
        searchMatches: [],
        searchActive: -1,
    };

    window.cytoscape.use(window.cytoscapeFcose);
    const cy = window.cytoscape({
        container: elements.graph,
        style: graphStyle(),
        minZoom: 0.05,
        maxZoom: 4,
    });

    //-------------------------------------------------------------------//
    // Settings
    //-------------------------------------------------------------------//

    // localStorage can be missing or throw (private windows, file:// in some browsers), so settings are best-effort.
    function loadSettings() {
        try {
            const stored = JSON.parse(window.localStorage.getItem(SETTINGS_KEY) || "{}");
            return { ...DEFAULT_SETTINGS, ...stored };
        } catch (error) {
            return { ...DEFAULT_SETTINGS };
        }
    }

    function saveSettings() {
        try {
            window.localStorage.setItem(SETTINGS_KEY, JSON.stringify(state.settings));
        } catch (error) {
            console.debug("Couldn't save settings:", error.message);
        }
    }

    function applySettingsToControls() {
        elements.depth.value = String(state.settings.depth);
        elements.direction.value = state.settings.direction;
        elements.hideHubs.checked = state.settings.hideHubs;
        elements.hubThreshold.value = String(state.settings.hubThreshold);
    }

    function readControls() {
        const threshold = Number.parseInt(elements.hubThreshold.value, 10);
        state.settings.depth = Number.parseInt(elements.depth.value, 10);
        state.settings.direction = elements.direction.value;
        state.settings.hideHubs = elements.hideHubs.checked;
        state.settings.hubThreshold = Number.isFinite(threshold) && threshold >= 2 ? threshold : DEFAULT_SETTINGS.hubThreshold;
        saveSettings();
    }

    //-------------------------------------------------------------------//
    // Status and theme
    //-------------------------------------------------------------------//

    function setStatus(message, isError = false) {
        elements.status.textContent = message;
        elements.status.classList.toggle("error", isError);
    }

    function cssVar(name) {
        return getComputedStyle(document.documentElement).getPropertyValue(name).trim();
    }

    function graphStyle() {
        return [
            {
                selector: "node",
                style: {
                    "background-color": cssVar("--node"),
                    "label": "data(id)",
                    "color": cssVar("--node-text"),
                    "font-size": 11,
                    "text-valign": "bottom",
                    "text-margin-y": 3,
                    "text-background-color": cssVar("--bg"),
                    "text-background-opacity": 0.7,
                    "text-background-padding": 1,
                    "min-zoomed-font-size": 7,
                    "width": "mapData(degree, 0, 120, 14, 46)",
                    "height": "mapData(degree, 0, 120, 14, 46)",
                },
            },
            { selector: "node.hub", style: { "background-color": cssVar("--node-hub") } },
            {
                selector: "node.missing",
                style: {
                    "background-opacity": 0,
                    "border-width": 2,
                    "border-style": "dashed",
                    "border-color": cssVar("--node-missing"),
                },
            },
            { selector: "node.path", style: { "background-color": cssVar("--node-path") } },
            { selector: "node.focus", style: { "background-color": cssVar("--node-focus"), "font-weight": "bold" } },
            { selector: "node:selected", style: { "border-width": 3, "border-color": cssVar("--edge-highlight"), "border-style": "solid" } },
            {
                selector: "edge",
                style: {
                    "width": "mapData(count, 1, 6, 1, 4)",
                    "line-color": cssVar("--edge"),
                    "target-arrow-color": cssVar("--edge"),
                    "target-arrow-shape": "triangle",
                    "arrow-scale": 0.8,
                    "curve-style": "bezier",
                    "font-size": 10,
                    "color": cssVar("--text"),
                    "text-background-color": cssVar("--bg"),
                    "text-background-opacity": 0.85,
                    "text-background-padding": 2,
                    "text-rotation": "autorotate",
                    "text-wrap": "wrap",
                },
            },
            { selector: "edge.path", style: { "line-color": cssVar("--node-path"), "target-arrow-color": cssVar("--node-path"), "width": 3 } },
            {
                selector: "edge.highlight, edge:selected",
                style: { "line-color": cssVar("--edge-highlight"), "target-arrow-color": cssVar("--edge-highlight"), "label": "data(label)", "z-index": 10 },
            },
            { selector: ".faded", style: { "opacity": 0.15 } },
        ];
    }

    window.matchMedia("(prefers-color-scheme: dark)").addEventListener("change", () => cy.style(graphStyle()));

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
            || available.find((game) => game.key === state.settings.game)
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
        try {
            const graph = await state.source.graph(game);
            state.game = game;
            state.index = new GraphIndex(graph);
            state.detailsCache.clear();
            if (state.source.kind === "api") {
                state.settings.game = game;
                saveSettings();
            }
        } catch (error) {
            setStatus(`Couldn't load the schema graph: ${error.message}`, true);
            return;
        }

        elements.emptyState.hidden = true;
        elements.search.disabled = false;
        elements.relayout.disabled = false;
        elements.showAll.disabled = false;
        elements.tableNames.replaceChildren(...[...state.index.nodes.keys()].map((id) => {
            const option = document.createElement("option");
            option.value = id;
            return option;
        }));

        const nodeCount = [...state.index.nodes.values()].filter((node) => !node.missing).length;
        const edgeCount = [...state.index.outgoing.values()].reduce((sum, targets) => sum + [...targets.values()].reduce((s, e) => s + e.length, 0), 0);
        setStatus(`${nodeCount} tables, ${edgeCount} references. Search for a table to start.`);

        const hashTable = readHash().table;
        if (hashTable && state.index.nodes.has(hashTable)) {
            focusTable(hashTable);
        } else {
            cy.elements().remove();
            showPlaceholderDetails();
            elements.search.focus();
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

    function focusTable(id) {
        if (!state.index.nodes.has(id)) {
            return;
        }
        readControls();
        const { ids, truncated } = state.index.neighbourhood(id, {
            depth: state.settings.depth,
            direction: state.settings.direction,
            hubThreshold: state.settings.hubThreshold,
            hideHubs: state.settings.hideHubs,
            maxNodes: MAX_VISIBLE_NODES,
        });
        state.focus = id;
        state.path = null;
        state.visible = ids;
        renderGraph();
        selectTable(id);
        writeHash();

        const hidden = truncated ? ` Showing the first ${MAX_VISIBLE_NODES}; lower the depth or hide hubs to see the rest.` : "";
        setStatus(`${ids.size} tables around ${id}.${hidden}`);
    }

    // Adds a table (and, optionally, its neighbours) to the current view without changing the focus.
    function addToView(ids) {
        const before = state.visible.size;
        for (const id of ids) {
            if (state.visible.size >= MAX_VISIBLE_NODES) {
                break;
            }
            state.visible.add(id);
        }
        if (state.visible.size !== before) {
            renderGraph();
        }
    }

    function showAll() {
        readControls();
        const ids = [...state.index.nodes.keys()].filter((id) => !(state.settings.hideHubs && state.index.degree(id) > state.settings.hubThreshold));
        if (ids.length > 600 && !window.confirm(`Draw all ${ids.length} tables? The layout can take a while and the graph will be dense.`)) {
            return;
        }
        state.focus = null;
        state.path = null;
        state.visible = new Set(ids);
        renderGraph({ quality: "draft" });
        setStatus(`Showing all ${ids.length} tables.`);
    }

    function renderGraph(options = {}) {
        const pathEdges = new Set();
        if (state.path) {
            for (let i = 0; i + 1 < state.path.length; i++) {
                pathEdges.add(`${state.path[i]}|${state.path[i + 1]}`);
                pathEdges.add(`${state.path[i + 1]}|${state.path[i]}`);
            }
        }
        const pathNodes = new Set(state.path || []);

        const elementsToAdd = [];
        for (const id of state.visible) {
            const node = state.index.nodes.get(id);
            const classes = [];
            if (node.missing) {
                classes.push("missing");
            }
            if (state.index.degree(id) > state.settings.hubThreshold) {
                classes.push("hub");
            }
            if (pathNodes.has(id)) {
                classes.push("path");
            }
            if (id === state.focus) {
                classes.push("focus");
            }
            elementsToAdd.push({ group: "nodes", data: { id, degree: state.index.degree(id) }, classes: classes.join(" ") });
        }

        // One drawn edge per pair of tables and direction, labelled with all the columns behind it.
        for (const from of state.visible) {
            for (const [to, edges] of state.index.outgoing.get(from) || []) {
                if (!state.visible.has(to)) {
                    continue;
                }
                const label = edges.map((edge) => `${edge.from_column} -> ${edge.to_column}`).join("\n");
                elementsToAdd.push({
                    group: "edges",
                    data: { id: `${from}|${to}`, source: from, target: to, label, count: edges.length },
                    classes: pathEdges.has(`${from}|${to}`) ? "path" : "",
                });
            }
        }

        cy.batch(() => {
            cy.elements().remove();
            cy.add(elementsToAdd);
        });
        runLayout(options.quality || "default");
    }

    function runLayout(quality) {
        if (cy.nodes().length === 0) {
            return;
        }
        const layout = cy.layout({
            name: "fcose",
            quality,
            randomize: true,
            animate: cy.nodes().length < 150,
            animationDuration: 400,
            nodeRepulsion: 9000,
            idealEdgeLength: 90,
            nodeSeparation: 80,
            packComponents: true,
            fit: true,
            padding: 30,
        });

        // Fitting a handful of nodes zooms far in, which makes labels huge, so cap it.
        layout.one("layoutstop", () => {
            if (cy.zoom() > MAX_FIT_ZOOM) {
                cy.zoom(MAX_FIT_ZOOM);
                cy.center();
            }
        });
        layout.run();
    }

    cy.on("tap", "node", (event) => selectTable(event.target.id()));
    cy.on("dbltap", "node", (event) => focusTable(event.target.id()));
    cy.on("tap", "edge", (event) => {
        const edge = event.target;
        setStatus(`${edge.source().id()} references ${edge.target().id()}: ${edge.data("label").replace(/\n/g, ", ")}`);
    });
    cy.on("mouseover", "node", (event) => {
        const neighbourhood = event.target.closedNeighborhood();
        cy.batch(() => {
            cy.elements().not(neighbourhood).addClass("faded");
            neighbourhood.edges().addClass("highlight");
        });
    });
    cy.on("mouseout", "node", () => {
        cy.batch(() => cy.elements().removeClass("faded highlight"));
    });

    //-------------------------------------------------------------------//
    // Details panel
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
        const button = element("button", { text, className: "link", title: "Show this table (double-click a node to focus on it)" });
        button.type = "button";
        button.addEventListener("click", () => {
            addToView([id]);
            selectTable(id);
        });
        return button;
    }

    function actionButton(text, onClick, title) {
        const button = element("button", { text, title });
        button.type = "button";
        button.addEventListener("click", onClick);
        return button;
    }

    function showPlaceholderDetails() {
        elements.details.replaceChildren(element("p", {
            className: "muted",
            text: "Select a table to see its fields and references. Double-click a table to focus on it.",
        }));
    }

    async function selectTable(id) {
        state.selected = id;
        cy.$(":selected").unselect();
        cy.getElementById(id).select();

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
        if (state.selected === id) {
            renderDetails(id, details);
        }
    }

    function renderDetails(id, details) {
        const node = state.index.nodes.get(id);
        const degree = state.index.degree(id);
        const newest = details.definitions[0];

        const badges = element("div", { className: "badges" }, [
            details.missing ? element("span", { className: "badge badge-warning", text: "Not in schema" }) : null,
            newest ? element("span", { className: "badge", text: `v${newest.version}` }) : null,
            details.definitions.length > 1 ? element("span", { className: "badge", text: `${details.definitions.length} versions` }) : null,
            newest ? element("span", { className: "badge", text: `${newest.fields.length} fields` }) : null,
            node?.localised ? element("span", { className: "badge", text: "Localised" }) : null,
            element("span", { className: "badge", text: `${degree} linked tables` }),
            degree > state.settings.hubThreshold ? element("span", { className: "badge", text: "Hub" }) : null,
        ]);

        const actions = element("div", { className: "actions" }, [
            actionButton("Focus here", () => focusTable(id), "Show the tables around this one"),
            actionButton("Expand", () => {
                addToView(state.index.neighbours(id, state.settings.direction));
                selectTable(id);
            }, "Add this table's neighbours to the view"),
            actionButton("Copy name", () => copyText(details.table), "Copy the full table name"),
        ]);

        const sections = [element("h2", { text: details.table }), badges, actions];

        if (details.missing) {
            sections.push(element("p", {
                className: "muted",
                text: "Other tables reference this table, but the schema has no definition for it.",
            }));
        } else {
            sections.push(definitionSection(id, details));
        }

        sections.push(referencedBySection(details));
        sections.push(pathSection(id));
        elements.details.replaceChildren(...sections);
    }

    function definitionSection(id, details) {
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
            const path = state.index.shortestPath(fromId, toId, {
                direction: state.settings.direction,
                avoidHubsOver: state.settings.hideHubs ? state.settings.hubThreshold : null,
            });
            if (!path) {
                result.replaceChildren(element("p", { className: "muted", text: `No path from ${fromId} to ${toId} with the current direction and hub settings.` }));
                return;
            }
            showPath(path);
            result.replaceChildren(pathSteps(path));
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

    function showPath(path) {
        state.focus = path[0];
        state.path = path;
        state.visible = new Set(path);
        renderGraph();
        setStatus(`Path of ${path.length - 1} steps from ${path[0]} to ${path[path.length - 1]}.`);
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
        focusTable(node.id);
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
        if (state.focus) {
            params.set("table", state.focus);
        }
        const hash = `#${params.toString()}`;
        if (window.location.hash !== hash) {
            window.history.replaceState(null, "", hash);
        }
    }

    window.addEventListener("hashchange", () => {
        const { game, table } = readHash();
        if (game && state.source?.kind === "api" && game !== state.game) {
            elements.gameSelect.value = game;
            loadGame(game);
        } else if (table && state.index?.nodes.has(table) && table !== state.focus) {
            focusTable(table);
        }
    });

    //-------------------------------------------------------------------//
    // Controls, file picking and drag and drop
    //-------------------------------------------------------------------//

    function refocus() {
        readControls();
        if (state.focus && !state.path) {
            focusTable(state.focus);
        }
    }

    elements.depth.addEventListener("change", refocus);
    elements.direction.addEventListener("change", refocus);
    elements.hideHubs.addEventListener("change", refocus);
    elements.hubThreshold.addEventListener("change", refocus);
    elements.relayout.addEventListener("click", () => runLayout("default"));
    elements.showAll.addEventListener("click", showAll);
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

    applySettingsToControls();
    ApiSource.detect().then((source) => {
        if (source) {
            useSource(source).catch((error) => setStatus(`Couldn't load games: ${error.message}`, true));
        } else {
            setStatus("Open or drop a schema file to start.");
        }
    });
})();
