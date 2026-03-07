# Server

The RPFM Server (`rpfm_server`) is a standalone backend process that exposes a **WebSocket-based IPC protocol** for programmatic access to RPFM's functionality. Any language that supports WebSockets and JSON can communicate with the server — no Rust or Qt dependencies required on the client side.

This section documents the protocol, all available commands and responses, and provides client implementation examples.

## Connecting

The server listens on `ws://127.0.0.1:45127/ws` by default. Connect with any WebSocket client:

<!-- langtabs-start -->
```typescript
const ws = new WebSocket("ws://127.0.0.1:45127/ws");
```
```csharp
using var ws = new ClientWebSocket();
await ws.ConnectAsync(new Uri("ws://127.0.0.1:45127/ws"), CancellationToken.None);
```
<!-- langtabs-end -->

Upon connection, the server immediately sends a `SessionConnected` message (with `id: 0`) containing the session ID assigned to this connection:

```json
{ "id": 0, "data": { "SessionConnected": 42 } }
```

## Sessions

The server supports multiple concurrent sessions. Each session maintains its own state (open packs, game selection, etc.).

### Reconnection

To reconnect to an existing session, append `?session_id=<id>` to the WebSocket URL:

<!-- langtabs-start -->
```typescript
const ws = new WebSocket("ws://127.0.0.1:45127/ws?session_id=42");
```
```csharp
using var ws = new ClientWebSocket();
await ws.ConnectAsync(new Uri("ws://127.0.0.1:45127/ws?session_id=42"), CancellationToken.None);
```
<!-- langtabs-end -->

### Session Listing (REST)

A REST endpoint is available for listing active sessions:

```
GET http://127.0.0.1:45127/sessions
```

Returns a JSON array of session info objects:

```json
[
  {
    "session_id": 42,
    "connection_count": 1,
    "timeout_remaining_secs": null,
    "is_shutting_down": false,
    "pack_names": ["my_mod.pack"]
  }
]
```

### Disconnection

Send the `ClientDisconnecting` command before closing the WebSocket so the server can clean up resources immediately instead of waiting for a timeout:

```json
{ "id": 99, "data": "ClientDisconnecting" }
```

## Message Format

Every message — both commands (client to server) and responses (server to client) — is wrapped in a `Message` envelope:

```json
{
  "id": <number>,
  "data": <Command or Response>
}
```

| Field  | Type   | Description |
|--------|--------|-------------|
| `id`   | number | Unique request ID. The server echoes this in the response so the client can correlate requests and responses. Use `0` only for unsolicited server messages. |
| `data`  | object or string | The command or response payload. |

### Request-Response Correlation

The `id` field enables asynchronous communication. Multiple requests can be in flight simultaneously — match each response back to its request by `id`.

## Serialization Convention

All messages use JSON. Rust enums (which back both `Command` and `Response`) are serialized by [serde](https://serde.rs/) as follows:

| Rust Variant         | JSON Serialization                      | Example                                      |
|----------------------|-----------------------------------------|----------------------------------------------|
| Unit variant         | `"VariantName"`                         | `"NewPack"`                                  |
| Newtype variant      | `{ "VariantName": value }`              | `{ "ClosePack": "my_mod.pack" }`             |
| Tuple variant        | `{ "VariantName": [v1, v2, ...] }`      | `{ "SavePackAs": ["key", "/path/to/file"] }` |

## Quick Start

Here is a minimal example that connects to the server, opens a pack file, and prints the response:

<!-- langtabs-start -->
```typescript
const ws = new WebSocket("ws://127.0.0.1:45127/ws");

let nextId = 1;
let currentSessionId: number | null = null;

function send(command: object | string): number {
  const id = nextId++;
  ws.send(JSON.stringify({ id, data: command }));
  return id;
}

// Listen for responses
ws.onmessage = (event) => {
  const msg = JSON.parse(event.data);

  // Handle the SessionConnected message sent immediately after connection
  if (typeof msg.data === "object" && "SessionConnected" in msg.data) {
    currentSessionId = msg.data.SessionConnected;
    console.log(`Connected to session ${currentSessionId}`);
    return;
  }

  console.log(`Response for request ${msg.id}:`, msg.data);
};

// Open a pack file once connected
ws.onopen = () => {
  send({ OpenPackFiles: ["/path/to/my_mod.pack"] });
};
```
```csharp
using System.Net.WebSockets;
using System.Text;
using System.Text.Json;

using var ws = new ClientWebSocket();
await ws.ConnectAsync(new Uri("ws://127.0.0.1:45127/ws"), CancellationToken.None);

int nextId = 1;
int? currentSessionId = null;

async Task Send(JsonElement command)
{
    var id = nextId++;
    var msg = JsonSerializer.Serialize(new { id, data = command });
    var bytes = Encoding.UTF8.GetBytes(msg);
    await ws.SendAsync(bytes, WebSocketMessageType.Text, true, CancellationToken.None);
}

// Listen for responses
var buffer = new byte[4096];
while (ws.State == WebSocketState.Open)
{
    var result = await ws.ReceiveAsync(buffer, CancellationToken.None);
    var json = Encoding.UTF8.GetString(buffer, 0, result.Count);
    var msg = JsonDocument.Parse(json).RootElement;

    // Handle the SessionConnected message sent immediately after connection
    if (msg.GetProperty("data").ValueKind == JsonValueKind.Object
        && msg.GetProperty("data").TryGetProperty("SessionConnected", out var sessionId))
    {
        currentSessionId = sessionId.GetInt32();
        Console.WriteLine($"Connected to session {currentSessionId}");

        // Open a pack file once connected
        var openCmd = JsonSerializer.SerializeToElement(
            new { OpenPackFiles = new[] { "/path/to/my_mod.pack" } });
        await Send(openCmd);
        continue;
    }

    Console.WriteLine($"Response for request {msg.GetProperty("id")}: {msg.GetProperty("data")}");
}
```
<!-- langtabs-end -->
