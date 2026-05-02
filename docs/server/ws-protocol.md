# WebSocket protocol

This page covers the protocol layer: how messages are framed, serialized and correlated. The full vocabulary lives in [Shared types](./ws-shared-types.md), [Commands](./ws-commands.md) and [Responses](./ws-responses.md).

## Connecting

The server listens on `ws://127.0.0.1:45127/ws` by default. Append `?session_id=<id>` to reconnect to an existing session — see [Sessions](./sessions.md).

<!-- langtabs-start -->
```typescript
const ws = new WebSocket("ws://127.0.0.1:45127/ws");
```
```csharp
using var ws = new ClientWebSocket();
await ws.ConnectAsync(new Uri("ws://127.0.0.1:45127/ws"), CancellationToken.None);
```
<!-- langtabs-end -->

Immediately after the handshake the server pushes an unsolicited `SessionConnected` message with the session ID:

```json
{ "id": 0, "data": { "SessionConnected": 42 } }
```

## Message envelope

Every message — both directions — is wrapped in a `Message` envelope:

```json
{
  "id": <number>,
  "data": <Command or Response>
}
```

| Field  | Type             | Description |
|--------|------------------|-------------|
| `id`   | number           | Unique request ID. The server echoes it in the response. Use `0` only for unsolicited server-initiated messages. |
| `data` | object or string | The command or response payload. |

### Request-response correlation

The `id` lets multiple requests be in flight simultaneously. Match each response back to its originating request by `id`. A typical client maintains a `Map<id, { resolve, reject }>` of pending requests and resolves them as responses arrive.

## Serialization conventions

All messages are JSON. The Rust enums backing `Command` and `Response` are serialized by [serde](https://serde.rs/) with these rules:

| Rust variant shape   | JSON serialization                | Example                                     |
|----------------------|------------------------------------|---------------------------------------------|
| Unit variant         | `"VariantName"`                    | `"NewPack"`                                 |
| Newtype variant      | `{ "VariantName": value }`         | `{ "ClosePack": "my_mod.pack" }`            |
| Tuple variant        | `{ "VariantName": [v1, v2, …] }`   | `{ "SavePackAs": ["key", "/path/to/file"] }`|

Struct variants use `{ "VariantName": { field: value, … } }`.

This means the JSON shape closely mirrors how a Rust client would write the same request — handy when reading the [Commands](./ws-commands.md) reference.

## A complete round-trip

<!-- langtabs-start -->
```typescript
const ws = new WebSocket("ws://127.0.0.1:45127/ws");

let nextId = 1;
const pending = new Map<number, (resp: any) => void>();

function send(command: object | string): Promise<any> {
  const id = nextId++;
  return new Promise((resolve) => {
    pending.set(id, resolve);
    ws.send(JSON.stringify({ id, data: command }));
  });
}

ws.onmessage = (e) => {
  const msg = JSON.parse(e.data);
  const cb = pending.get(msg.id);
  if (cb) {
    pending.delete(msg.id);
    cb(msg.data);
  }
};

ws.onopen = async () => {
  const resp = await send({ OpenPackFiles: ["/path/to/my_mod.pack"] });
  console.log("opened:", resp);
};
```
```csharp
using System.Net.WebSockets;
using System.Text;
using System.Text.Json;

using var ws = new ClientWebSocket();
await ws.ConnectAsync(new Uri("ws://127.0.0.1:45127/ws"), CancellationToken.None);

int nextId = 1;

async Task SendAsync(object command)
{
    var id = nextId++;
    var msg = JsonSerializer.Serialize(new { id, data = command });
    await ws.SendAsync(
        Encoding.UTF8.GetBytes(msg),
        WebSocketMessageType.Text, true, CancellationToken.None);
}

await SendAsync(new { OpenPackFiles = new[] { "/path/to/my_mod.pack" } });
```
<!-- langtabs-end -->

For a fuller, production-shaped client see [Client example](./client-example.md).

## Errors

Errors come back as the `Error(String)` response variant:

```json
{ "id": 7, "data": { "Error": "Failed to open pack: …" } }
```

Treat any `Error` response as a rejection of the originating request. The error message is suitable to surface to a developer; for end-user UIs you'll typically want to wrap it with friendlier copy.

## Disconnecting cleanly

Send `ClientDisconnecting` before closing the socket so the server tears the session down immediately instead of waiting for the 5-minute timeout:

```json
{ "id": 99, "data": "ClientDisconnecting" }
```

After sending, you can close the WebSocket. The server doesn't reply.

## What's next

- [Shared types](./ws-shared-types.md) — every payload type referenced in `Command` / `Response`.
- [Commands](./ws-commands.md) — every variant the client can send.
- [Responses](./ws-responses.md) — every variant the server can send back.
