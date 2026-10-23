// Where the explorer gets its schema from.
//
// `WsSource` fetches the selected game's schema from `rpfm_server` over its `/ws` WebSocket. The page is
// served by the server (`http://127.0.0.1:45127/schema-explorer/#game=<game>`) and connects back to
// whatever host served it, so it follows the server's address.

(function (root, factory) {
    const schemaModel = root.RpfmSchemaModel || (typeof require === "function" ? require("./schema_model.js") : null);
    const api = factory(schemaModel);
    if (typeof module === "object" && module.exports) {
        module.exports = api;
    } else {
        root.RpfmDataSource = api;
    }
})(typeof self !== "undefined" ? self : this, (schemaModel) => {
    "use strict";

    /**
     * A schema in memory, with the graph and table details the explorer shows.
     */
    class SchemaSource {

        /**
         * @param {{key: string, label: string, schema: Object}} options `key` is the game key (used for the saved
         *   layout), `label` is shown to the user.
         */
        constructor({ key, label, schema }) {
            this.key = key;
            this.label = label;
            this.model = new schemaModel.SchemaModel(schema);
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

    /**
     * Fetch a game's schema from `rpfm_server` over its WebSocket, in a session of its own.
     *
     * Waits for the server's `SessionConnected` message, selects the game (without loading its dependencies),
     * asks for the schema and closes the socket. It never sends `ClientDisconnecting`, which would end the
     * session at once (and the server, if it was the last one): a plain close leaves it to expire on its own.
     *
     * @param {string} url The server's WebSocket URL.
     * @param {string|null} game The game key, or null to use the server's default game setting.
     * @param {(url: string) => WebSocket} [createSocket] Opens the socket. Replaceable for tests.
     * @returns {Promise<{game: string, schema: Object}>} The game key and its schema.
     */
    function fetchSchemaOverWebSocket(url, game, createSocket = (socketUrl) => new WebSocket(socketUrl)) {
        return new Promise((resolve, reject) => {
            const socket = createSocket(url);
            const pending = new Map();
            let nextId = 1;
            let started = false;
            let settled = false;

            const finish = (error, value) => {
                if (settled) {
                    return;
                }
                settled = true;
                socket.close();
                if (error) {
                    reject(error);
                } else {
                    resolve(value);
                }
            };

            // Requests are matched to replies by id. Replies carrying `Error` reject.
            const send = (data) => new Promise((onReply, onError) => {
                const id = nextId++;
                pending.set(id, { onReply, onError });
                socket.send(JSON.stringify({ id, data }));
            });

            const run = async () => {
                try {
                    let key = game;
                    if (!key) {
                        key = (await send({ SettingsGetString: "default_game" })).String;
                    }
                    if (!key) {
                        throw new Error("RPFM has no game selected.");
                    }
                    await send({ SetGameSelected: [key, false] });
                    const reply = await send("Schema");
                    finish(null, { game: key, schema: schemaModel.normalizeSchema(reply) });
                } catch (error) {
                    finish(error);
                }
            };

            socket.onmessage = (event) => {
                let message;
                try {
                    message = JSON.parse(event.data);
                } catch (error) {
                    return;
                }

                // Id 0 carries messages the server sends on its own: the session id first, then things like settings changes.
                if (message.id === 0) {
                    if (!started && message.data?.SessionConnected !== undefined) {
                        started = true;
                        run();
                    }
                    return;
                }
                const waiter = pending.get(message.id);
                if (!waiter) {
                    return;
                }
                pending.delete(message.id);
                if (message.data && typeof message.data === "object" && "Error" in message.data) {
                    waiter.onError(new Error(message.data.Error));
                } else {
                    waiter.onReply(message.data);
                }
            };
            socket.onerror = () => finish(new Error("Couldn't connect to rpfm_server."));
            socket.onclose = () => finish(new Error("rpfm_server closed the connection before sending the schema."));
        });
    }

    const WsSource = {

        /**
         * Check whether the page is served by `rpfm_server` (rather than opened from disk).
         *
         * @returns {boolean} Whether the schema can be fetched from the server.
         */
        isAvailable() {
            return window.location.protocol === "http:" || window.location.protocol === "https:";
        },

        /**
         * Fetch a game's schema from the server that served the page.
         *
         * @param {string|null} game The game key, or null for the server's default game.
         * @returns {Promise<SchemaSource>} The source.
         * @throws {Error} If the server can't be reached or has no schema for the game.
         */
        async load(game) {
            const scheme = window.location.protocol === "https:" ? "wss" : "ws";
            const { game: key, schema } = await fetchSchemaOverWebSocket(`${scheme}://${window.location.host}/ws`, game);
            return new SchemaSource({ key, label: key, schema });
        },
    };

    return { SchemaSource, WsSource, fetchSchemaOverWebSocket };
});
