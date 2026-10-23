// Where the explorer gets its schema data from.
//
// - `ApiSource` reads the planned `rpfm_server` endpoints (`api/games`, `api/{game}/graph`,
//   `api/{game}/tables/{table}`, see docs/server/schema_explorer.md), relative to the page.
// - `FileSource` builds the same data in the browser from a schema file the user loads.
//
// Both expose the same async methods, so the UI doesn't care which one it uses.

(function (root, factory) {
    root.RpfmDataSource = factory(root.RpfmSchemaModel);
})(typeof self !== "undefined" ? self : this, (schemaModel) => {
    "use strict";

    const API_PROBE_TIMEOUT_MS = 1500;

    // Resolve `api/` next to the page, whether it's served as `/schema-explorer`, `/schema-explorer/` or `.../index.html`.
    function apiBaseUrl() {
        const url = new URL(window.location.href);
        url.hash = "";
        url.search = "";
        if (url.pathname.endsWith(".html")) {
            url.pathname = url.pathname.slice(0, url.pathname.lastIndexOf("/") + 1);
        } else if (!url.pathname.endsWith("/")) {
            url.pathname += "/";
        }
        return new URL("api/", url);
    }

    async function fetchJson(url, timeoutMs) {
        const controller = new AbortController();
        const timer = timeoutMs ? setTimeout(() => controller.abort(), timeoutMs) : null;
        try {
            const response = await fetch(url, { signal: controller.signal, headers: { Accept: "application/json" } });
            if (!response.ok) {
                throw new Error(`HTTP ${response.status} for ${url.pathname}`);
            }
            return await response.json();
        } finally {
            if (timer) {
                clearTimeout(timer);
            }
        }
    }

    class ApiSource {
        constructor(baseUrl, games) {
            this.baseUrl = baseUrl;
            this.gameList = games;
            this.kind = "api";
        }

        /**
         * Check whether the page is served by an `rpfm_server` with the explorer API.
         *
         * @returns {Promise<ApiSource|null>} The source, or null if there's no API (e.g. opened from disk).
         */
        static async detect() {
            if (window.location.protocol !== "http:" && window.location.protocol !== "https:") {
                return null;
            }
            try {
                const baseUrl = apiBaseUrl();
                const games = await fetchJson(new URL("games", baseUrl), API_PROBE_TIMEOUT_MS);
                return Array.isArray(games) ? new ApiSource(baseUrl, games) : null;
            } catch (error) {
                console.info("Schema explorer API not available, using file mode:", error.message);
                return null;
            }
        }

        async games() {
            return this.gameList;
        }

        async graph(game) {
            return fetchJson(new URL(`${encodeURIComponent(game)}/graph`, this.baseUrl));
        }

        async tableDetails(game, table) {
            return fetchJson(new URL(`${encodeURIComponent(game)}/tables/${encodeURIComponent(table)}`, this.baseUrl));
        }
    }

    class FileSource {
        constructor(fileName, schema) {
            this.fileName = fileName;
            this.model = new schemaModel.SchemaModel(schema);
            this.kind = "file";
        }

        /**
         * Build a source from the text of a schema file.
         *
         * @param {string} fileName The file name, shown as the "game".
         * @param {string} text The file contents (RON or JSON).
         * @returns {FileSource} The source.
         * @throws {Error} If the file isn't a schema.
         */
        static fromText(fileName, text) {
            return new FileSource(fileName, schemaModel.parseSchemaText(text));
        }

        async games() {
            return [{ key: this.fileName, display_name: this.fileName, schema_available: true }];
        }

        async graph() {
            return this.model.graph();
        }

        async tableDetails(_game, table) {
            const details = this.model.tableDetails(table);
            if (!details) {
                throw new Error(`Table '${table}' isn't in this schema.`);
            }
            return details;
        }
    }

    return { ApiSource, FileSource };
});
