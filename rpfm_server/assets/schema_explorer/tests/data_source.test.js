// Run with: node --test "rpfm_server/assets/schema_explorer/tests/*.test.js"

const test = require("node:test");
const assert = require("node:assert/strict");

const { fetchSchemaOverWebSocket } = require("../js/data_source.js");

const SCHEMA = {
    version: 5,
    definitions: { battles_tables: [{ version: 1, fields: [], localised_fields: [], localised_key_order: [] }] },
    patches: {},
};

// A stand-in WebSocket: records what the client sends, and answers each request through `respond`.
function fakeSocket(respond) {
    const socket = {
        sent: [],
        closed: false,
        send(text) {
            const message = JSON.parse(text);
            socket.sent.push(message.data);
            const reply = respond(message.data);
            if (reply !== undefined) {
                setTimeout(() => socket.receive({ id: message.id, data: reply }), 0);
            }
        },
        close() {
            socket.closed = true;
        },
        receive(message) {
            socket.onmessage({ data: JSON.stringify(message) });
        },
    };
    return socket;
}

// Opens a fake socket and has the server greet the client like rpfm_server does.
function connect(respond) {
    let socket;
    const createSocket = () => {
        socket = fakeSocket(respond);
        setTimeout(() => {
            socket.receive({ id: 0, data: { SettingsChanged: { bool: {} } } });
            socket.receive({ id: 0, data: { SessionConnected: 7 } });
        }, 0);
        return socket;
    };
    return { createSocket, socket: () => socket };
}

function server(data) {
    if (data === "Schema") {
        return { Schema: SCHEMA };
    }
    if (data.SetGameSelected) {
        return { CompressionFormatDependenciesInfo: ["None", null] };
    }
    if (data.SettingsGetString === "default_game") {
        return { String: "warhammer_2" };
    }
    return undefined;
}

test("selects the game, asks for the schema, and closes without ClientDisconnecting", async () => {
    const connection = connect(server);

    const result = await fetchSchemaOverWebSocket("ws://127.0.0.1:45127/ws", "warhammer_3", connection.createSocket);

    assert.equal(result.game, "warhammer_3");
    assert.deepEqual(Object.keys(result.schema.definitions), ["battles_tables"]);
    assert.deepEqual(connection.socket().sent, [{ SetGameSelected: ["warhammer_3", false] }, "Schema"]);
    assert.equal(connection.socket().closed, true);
    assert.ok(!connection.socket().sent.includes("ClientDisconnecting"));
});

test("uses the server's default game when none is given", async () => {
    const connection = connect(server);

    const result = await fetchSchemaOverWebSocket("ws://127.0.0.1:45127/ws", null, connection.createSocket);

    assert.equal(result.game, "warhammer_2");
    assert.deepEqual(connection.socket().sent[0], { SettingsGetString: "default_game" });
    assert.deepEqual(connection.socket().sent[1], { SetGameSelected: ["warhammer_2", false] });
});

test("reports the server's errors", async () => {
    const connection = connect((data) => (data === "Schema" ? { Error: "There is no Schema for the Game Selected." } : server(data)));

    await assert.rejects(
        fetchSchemaOverWebSocket("ws://127.0.0.1:45127/ws", "arena", connection.createSocket),
        /There is no Schema for the Game Selected/,
    );
    assert.equal(connection.socket().closed, true);
});

test("reports a connection that fails or closes early", async () => {
    let socket;
    const failing = () => {
        socket = fakeSocket(() => undefined);
        setTimeout(() => socket.onerror(), 0);
        return socket;
    };

    await assert.rejects(fetchSchemaOverWebSocket("ws://127.0.0.1:45127/ws", "warhammer_3", failing), /Couldn't connect/);
});
