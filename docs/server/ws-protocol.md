# WebSocket protocol

The `/ws` endpoint speaks [JSON-RPC 2.0][jsonrpc]: every WebSocket text frame is one JSON-RPC request from the client, or one response or notification from the server. The methods themselves are listed in [Methods](./methods.md).

[jsonrpc]: https://www.jsonrpc.org/specification

## Connecting

Open a WebSocket to `ws://127.0.0.1:45127/ws`. To reattach to an existing session, pass its ID: `ws://127.0.0.1:45127/ws?session_id=12345` (see [Sessions](./sessions.md)).

Right after connecting, the server sends a `session.connected` notification with the ID of your session:

```json
{ "jsonrpc": "2.0", "method": "session.connected", "params": { "session_id": 12345 } }
```

Store it if you want to reconnect to the same session later.

## Requests and responses

A request names a method and passes its params as an object:

```json
{ "jsonrpc": "2.0", "id": 7, "method": "table.rows", "params": { "file": { "source": { "pack": "0b4c5a62-7d3e-4f1a-9c2b-8e6d5f4a3b21" }, "path": "db/units_tables/my_units" }, "limit": 2 } }
```

The response carries the same `id`, and either a `result` or an `error`:

```json
{ "jsonrpc": "2.0", "id": 7, "result": { "columns": ["key", "..."], "rows": [ ... ], "total": 51 } }
```

Use a different `id` for each request: responses arrive in the order requests finish, not the order they were sent. Requests of the same session still **run** in the order they arrived, one at a time, so a `pack.save` sent before a `pack.close` always saves the pack before closing it.

Methods without params can omit `params`.

## Errors

Failed requests get an `error` with a code, a message, and a `data` object with the kind of error, so clients can match on it without parsing the message:

```json
{ "jsonrpc": "2.0", "id": 8, "error": { "code": -32001, "message": "Pack not found: nope.pack", "data": { "kind": "pack_not_found", "details": "nope.pack" } } }
```

| Code     | `kind`                    | Meaning                                                            |
|----------|---------------------------|--------------------------------------------------------------------|
| `-32001` | `pack_not_found`          | No open pack has that key.                                         |
| `-32002` | `file_not_found`          | The file doesn't exist in its source.                              |
| `-32003` | `not_a_table`             | The file isn't a DB or Loc table.                                  |
| `-32004` | `schema_not_loaded`       | The selected game has no schema loaded.                            |
| `-32005` | `dependencies_not_loaded` | The dependencies of the selected game aren't loaded.               |
| `-32006` | `definition_not_found`    | The schema has no definition for the table.                        |
| `-32007` | `read_only`               | The source can't be edited, like the game files.                   |
| `-32008` | `job_not_found`           | No job has that ID.                                                |
| `-32009` | `diagnostics_not_run`     | `diagnostics.list` or `diagnostics.report` before any check.      |
| `-32010` | `search_not_run`          | `search.matches`, `search.replace` or `search.report` before any search. |
| `-32011` | `not_found`               | Something the request names doesn't exist.                         |
| `-32601` | `method_not_found`        | Unknown method.                                                    |
| `-32602` | `invalid_params`          | The params don't match the method (including unknown params), or a value is invalid. |
| `-32603` | `internal`                | Anything else.                                                     |

## Jobs

Methods that can take a long time (selecting a game, generating the dependencies cache, diagnostics, search, schema updates, the optimizer, Lua tests) run as **jobs**. They answer right away with the ID of their job:

```json
{ "jsonrpc": "2.0", "id": 9, "result": { "job": 3 } }
```

The job then runs in order with the other requests of the session, and every change of its state is sent as a `job.updated` notification:

```json
{ "jsonrpc": "2.0", "method": "job.updated", "params": { "job": 3, "method": "session.set_game", "state": "running", "stage": "Loading the schema and the dependencies" } }
{ "jsonrpc": "2.0", "method": "job.updated", "params": { "job": 3, "method": "session.set_game", "state": "finished", "result": { ... } } }
```

A job is `queued`, `running` (with an optional `stage`, and an optional `progress` from 0 to 100), `finished` (with the `result` of its method), `failed` (with its `error`) or `cancelled`. Instead of following the notifications, you can also call `job.wait`, which answers when the job ends or after a timeout, or `job.status`. Queued jobs can be cancelled with `job.cancel`; running ones can't.

Diagnostics checks (`diagnostics.run`) don't make other requests wait: a running check stops when another request arrives, and goes back to the queue behind it. A new check replaces the previous one if it hasn't ended, which ends `cancelled`, and also checks what the replaced one would have checked. Follow the job of the last check you started to get its results.

## Conventions

- **Pack keys.** Open packs are identified by the `key` that `pack.open`, `pack.new` and `session.status` return: a UUID, like `0b4c5a62-7d3e-4f1a-9c2b-8e6d5f4a3b21`. Keys never change while a pack is open, even when it's saved under another name, and aren't reused. A pack's `name` is its file name, and its `path` is only set if it's saved on disk. A method's `pack` param is always the open pack it works on or changes. Where it's optional, like in lookups and lists, setting it limits the method to that pack.
- **Sources.** Files are found in a `source`: `{ "pack": "<key>" }` for an open pack, or `"game_files"`, `"parent_files"` or `"assembly_kit"` for the dependencies. A file is a `{ "source": ..., "path": ... }` pair.
- **Reading and writing.** Methods reading files take a `source` or a `file`, so they can read from anywhere. Methods changing files take a `pack` and a `path`, as only the files of open packs can change.
- **Paths.** Paths inside packs use `/`. Where a method takes files or folders, a path is a file if one exists with that path, and a folder otherwise. Filters by path are called `path_prefix`. Paths on disk have names saying what they are, like `destination` or `tsv_path`.
- **Table names.** Tables are named by their full name, like `units_tables`, in params (`table_name`) and in results, including the references between columns.
- **Pagination.** Lists take an `offset` and an optional `limit`, and include the `total` amount of items, so you never get more than you asked for.
- **Settings.** Sessions start with the settings in RPFM's `settings.json`. Clients with their own settings, like the UI, send them with `session.configure`.

## A complete round-trip

<!-- langtabs-start -->
```typescript
const ws = new WebSocket("ws://127.0.0.1:45127/ws");
let nextId = 1;
const pending = new Map<number, (message: any) => void>();

function call(method: string, params?: object): Promise<any> {
  const id = nextId++;
  ws.send(JSON.stringify({ jsonrpc: "2.0", id, method, params }));
  return new Promise((resolve, reject) => pending.set(id, (message) => message.error ? reject(message.error) : resolve(message.result)));
}

ws.onmessage = (event) => {
  const message = JSON.parse(event.data);
  if ("id" in message) {
    pending.get(message.id)?.(message);
    pending.delete(message.id);
  } else if (message.method === "session.connected") {
    console.log("session", message.params.session_id);
  }
};

ws.onopen = async () => {
  const pack = await call("pack.open", { paths: ["/path/to/my_mod.pack"] });
  const files = await call("files.list", { source: { pack: pack.key }, path_prefix: "db/", limit: 20 });
  console.log(files.total, files.files);
};
```
```csharp
using var ws = new ClientWebSocket();
await ws.ConnectAsync(new Uri("ws://127.0.0.1:45127/ws"), CancellationToken.None);

var request = JsonSerializer.Serialize(new {
    jsonrpc = "2.0",
    id = 1,
    method = "pack.open",
    @params = new { paths = new[] { "/path/to/my_mod.pack" } },
});

await ws.SendAsync(Encoding.UTF8.GetBytes(request), WebSocketMessageType.Text, true, CancellationToken.None);

// Read messages until the one with "id": 1 arrives. The first one is the session.connected notification.
```
<!-- langtabs-end -->

## Disconnecting cleanly

Call `session.disconnect` before closing the socket. The server removes your session right away instead of keeping it for 5 minutes, and exits if it was the last one:

```json
{ "jsonrpc": "2.0", "id": 99, "method": "session.disconnect" }
```

## What's next

- [Methods](./methods.md) — every method, with its request and response types.
- [Sessions](./sessions.md) — reconnection and session lifetime.
- [Client example](./client-example.md) — a complete client with typed helpers.
