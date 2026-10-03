# Client Example

This page provides a complete client implementation example. The same patterns apply to any language with WebSocket and JSON support. See the [WebSocket protocol](./ws-protocol.md) for the message format, and [Methods](./methods.md) for the params and results of each method.

## The Client

The client below wraps the WebSocket connection, matches responses to their requests by `id`, waits for jobs, and provides typed methods for common operations.

### Connection and Requests

<!-- langtabs-start -->
```typescript
interface RpcError {
  code: number;
  message: string;
  data?: { kind: string; details?: string };
}

type JobStatus =
  { job: number; method: string } & (
    | { state: "queued" }
    | { state: "running"; stage?: string }
    | { state: "finished"; result: any }
    | { state: "failed"; error: RpcError }
    | { state: "cancelled" }
  );

class RpfmError extends Error {
  constructor(public error: RpcError) {
    super(error.message);
  }

  get kind(): string | undefined {
    return this.error.data?.kind;
  }
}

class RpfmClient {
  private ws: WebSocket;
  private nextId = 1;
  private pending = new Map<number, { resolve: (result: any) => void; reject: (error: Error) => void }>();
  public sessionId: number | null = null;

  /** Called with every `job.updated` notification, to show progress. */
  public onJobUpdated: (status: JobStatus) => void = () => {};

  private constructor(ws: WebSocket) {
    this.ws = ws;
    this.ws.onmessage = (event) => this.handleMessage(JSON.parse(event.data));
    this.ws.onclose = () => {
      for (const { reject } of this.pending.values()) {
        reject(new Error("Connection closed"));
      }
      this.pending.clear();
    };
  }

  /** Connects to the server, optionally reattaching to an existing session. */
  static connect(url = "ws://127.0.0.1:45127/ws", sessionId?: number): Promise<RpfmClient> {
    const fullUrl = sessionId !== undefined ? `${url}?session_id=${sessionId}` : url;
    const client = new RpfmClient(new WebSocket(fullUrl));

    // The connection is ready once the server tells us our session ID.
    return new Promise((resolve, reject) => {
      client.ws.addEventListener("message", function onConnected(event) {
        const message = JSON.parse(event.data);
        if (message.method === "session.connected") {
          client.ws.removeEventListener("message", onConnected);
          resolve(client);
        }
      });
      client.ws.addEventListener("error", () => reject(new Error("Connection failed")));
    });
  }

  private handleMessage(message: any) {
    if ("id" in message) {
      const pending = this.pending.get(message.id);
      this.pending.delete(message.id);
      if (message.error) {
        pending?.reject(new RpfmError(message.error));
      } else {
        pending?.resolve(message.result);
      }
    } else if (message.method === "session.connected") {
      this.sessionId = message.params.session_id;
    } else if (message.method === "job.updated") {
      this.onJobUpdated(message.params);
    }
  }

  /** Calls a method and returns its result. Fails with an `RpfmError` if the method fails. */
  call<T = any>(method: string, params?: object): Promise<T> {
    const id = this.nextId++;
    return new Promise((resolve, reject) => {
      this.pending.set(id, { resolve, reject });
      this.ws.send(JSON.stringify({ jsonrpc: "2.0", id, method, params }));
    });
  }

  /** Calls a method that runs as a job, and returns its result once the job ends. */
  async runJob<T = any>(method: string, params?: object): Promise<T> {
    const { job } = await this.call<{ job: number }>(method, params);
    while (true) {
      const status = await this.call<JobStatus>("job.wait", { job, timeout_secs: 60 });
      switch (status.state) {
        case "finished": return status.result;
        case "failed": throw new RpfmError(status.error);
        case "cancelled": throw new Error(`Job ${job} was cancelled`);
        case "queued":
        case "running": break;
      }
    }
  }

  /** Ends the session and closes the connection. */
  async disconnect(): Promise<void> {
    await this.call("session.disconnect");
    this.ws.close();
  }
```
```csharp
using System.Collections.Concurrent;
using System.Net.WebSockets;
using System.Text;
using System.Text.Json;
using System.Text.Json.Nodes;

public class RpfmException : Exception
{
    public int Code { get; }
    public string? Kind { get; }

    public RpfmException(JsonNode error) : base(error["message"]?.GetValue<string>())
    {
        Code = error["code"]!.GetValue<int>();
        Kind = error["data"]?["kind"]?.GetValue<string>();
    }
}

public class RpfmClient : IAsyncDisposable
{
    private readonly ClientWebSocket _ws = new();
    private readonly ConcurrentDictionary<long, TaskCompletionSource<JsonNode?>> _pending = new();
    private readonly TaskCompletionSource _connected = new();
    private long _nextId = 1;

    public long? SessionId { get; private set; }

    /// Called with every `job.updated` notification, to show progress.
    public event Action<JsonNode>? JobUpdated;

    /// Connects to the server, optionally reattaching to an existing session.
    public static async Task<RpfmClient> ConnectAsync(string url = "ws://127.0.0.1:45127/ws", long? sessionId = null)
    {
        var client = new RpfmClient();
        var fullUrl = sessionId is null ? url : $"{url}?session_id={sessionId}";
        await client._ws.ConnectAsync(new Uri(fullUrl), CancellationToken.None);
        _ = client.ReceiveLoopAsync();

        // The connection is ready once the server tells us our session ID.
        await client._connected.Task;
        return client;
    }

    private async Task ReceiveLoopAsync()
    {
        var buffer = new byte[64 * 1024];
        var message = new MemoryStream();
        while (_ws.State == WebSocketState.Open)
        {
            var result = await _ws.ReceiveAsync(buffer, CancellationToken.None);
            if (result.MessageType == WebSocketMessageType.Close) break;

            message.Write(buffer, 0, result.Count);
            if (!result.EndOfMessage) continue;

            HandleMessage(JsonNode.Parse(message.ToArray())!);
            message.SetLength(0);
        }

        foreach (var pending in _pending.Values)
            pending.TrySetException(new Exception("Connection closed"));
    }

    private void HandleMessage(JsonNode message)
    {
        if (message["id"] is JsonNode id)
        {
            if (!_pending.TryRemove(id.GetValue<long>(), out var pending)) return;
            if (message["error"] is JsonNode error)
                pending.SetException(new RpfmException(error));
            else
                pending.SetResult(message["result"]);
        }
        else if (message["method"]?.GetValue<string>() == "session.connected")
        {
            SessionId = message["params"]!["session_id"]!.GetValue<long>();
            _connected.TrySetResult();
        }
        else if (message["method"]?.GetValue<string>() == "job.updated")
        {
            JobUpdated?.Invoke(message["params"]!);
        }
    }

    /// Calls a method and returns its result. Throws an `RpfmException` if the method fails.
    public async Task<JsonNode?> CallAsync(string method, object? @params = null)
    {
        var id = Interlocked.Increment(ref _nextId);
        var pending = new TaskCompletionSource<JsonNode?>(TaskCreationOptions.RunContinuationsAsynchronously);
        _pending[id] = pending;

        var request = JsonSerializer.SerializeToUtf8Bytes(new { jsonrpc = "2.0", id, method, @params });
        await _ws.SendAsync(request, WebSocketMessageType.Text, true, CancellationToken.None);
        return await pending.Task;
    }

    /// Calls a method that runs as a job, and returns its result once the job ends.
    public async Task<JsonNode?> RunJobAsync(string method, object? @params = null)
    {
        var job = (await CallAsync(method, @params))!["job"]!.GetValue<long>();
        while (true)
        {
            var status = (await CallAsync("job.wait", new { job, timeout_secs = 60 }))!;
            switch (status["state"]!.GetValue<string>())
            {
                case "finished": return status["result"];
                case "failed": throw new RpfmException(status["error"]!);
                case "cancelled": throw new Exception($"Job {job} was cancelled");
            }
        }
    }

    /// Ends the session and closes the connection.
    public async ValueTask DisposeAsync()
    {
        await CallAsync("session.disconnect");
        await _ws.CloseAsync(WebSocketCloseStatus.NormalClosure, null, CancellationToken.None);
    }
```
<!-- langtabs-end -->

### Typed Methods

Wrapping the methods you use gives you typed params and results. A few common ones:

<!-- langtabs-start -->
```typescript
  setGame(game: string): Promise<SessionStatus> {
    return this.runJob("session.set_game", { game });
  }

  openPack(paths: string[]): Promise<PackSummary> {
    return this.call("pack.open", { paths });
  }

  listFiles(pack: string, prefix = "", limit?: number): Promise<FileList> {
    return this.call("files.list", { source: { pack }, prefix, limit });
  }

  tableInfo(file: FileRef): Promise<TableInfo> {
    return this.call("table.info", { file });
  }

  tableRows(file: FileRef, options: { columns?: string[]; filters?: RowFilter[]; offset?: number; limit?: number } = {}): Promise<TableRows> {
    return this.call("table.rows", { file, ...options });
  }

  editTable(pack: string, path: string, edits: RowEdit[]): Promise<{ row_count: number }> {
    return this.call("table.edit", { pack, path, edits });
  }

  readText(file: FileRef): Promise<string> {
    return this.call("file.read", { file, format: "text" }).then((file) => file.contents.text);
  }

  writeText(pack: string, path: string, text: string): Promise<void> {
    return this.call("file.write", { pack, path, contents: { kind: "text", text } });
  }

  savePack(pack: string, path?: string): Promise<PackSummary> {
    return this.call("pack.save", { pack, path });
  }
}

type FileSource = { pack: string } | "game_files" | "parent_files" | "assembly_kit";

interface FileRef {
  source: FileSource;
  path: string;
}

interface PackSummary {
  key: string;
  name: string;
  path: string;
  pack_type: string;
  file_count: number;
}

interface SessionStatus {
  game: string;
  schema_loaded: boolean;
  packs: PackSummary[];
}

interface FileList {
  files: { path: string; file_type: string }[];
  folders: string[];
  total: number;
}

interface TableInfo {
  table_name: string;
  version: number;
  columns: { name: string }[];
  row_count: number;
}

interface RowFilter {
  column: string;
  op: "equals" | "not_equals" | "contains";
  value: string;
  ignore_case?: boolean;
}

interface TableRows {
  columns: string[];
  rows: { index: number; values: (boolean | number | string)[] }[];
  total: number;
}

type RowEdit =
  | { op: "insert"; index?: number; values?: Record<string, boolean | number | string> }
  | { op: "update"; index: number; values: Record<string, boolean | number | string> }
  | { op: "delete"; indexes: number[] };
```
```csharp
    public async Task<string> OpenPackAsync(string path) =>
        (await CallAsync("pack.open", new { paths = new[] { path } }))!["key"]!.GetValue<string>();

    public Task<JsonNode?> SetGameAsync(string game) =>
        RunJobAsync("session.set_game", new { game });

    public Task<JsonNode?> TableRowsAsync(object file, object[]? filters = null, int? limit = null) =>
        CallAsync("table.rows", new { file, filters = filters ?? [], limit });

    public Task<JsonNode?> EditTableAsync(string pack, string path, params object[] edits) =>
        CallAsync("table.edit", new { pack, path, edits });

    public Task<JsonNode?> SavePackAsync(string pack) =>
        CallAsync("pack.save", new { pack });

    public static object PackFile(string pack, string path) =>
        new { source = new { pack }, path };
}
```
<!-- langtabs-end -->

### Usage Example

<!-- langtabs-start -->
```typescript
const client = await RpfmClient.connect();
client.onJobUpdated = (status) => {
  if (status.state === "running" && status.stage) {
    console.log(`${status.method}: ${status.stage}`);
  }
};

await client.setGame("warhammer_3");
const pack = await client.openPack(["/path/to/my_mod.pack"]);

const tables = await client.listFiles(pack.key, "db/");
console.log(`${tables.total} tables`, tables.files.map((file) => file.path));

// Double the men of a unit.
const units: FileRef = { source: { pack: pack.key }, path: "db/land_units_tables/my_mod" };
const page = await client.tableRows(units, {
  columns: ["key", "num_men"],
  filters: [{ column: "key", op: "equals", value: "my_unit" }],
});

for (const row of page.rows) {
  const numMen = row.values[1] as number;
  await client.editTable(pack.key, units.path, [{ op: "update", index: row.index, values: { num_men: numMen * 2 } }]);
}

try {
  await client.tableRows({ source: { pack: pack.key }, path: "db/missing_tables/data" });
} catch (error) {
  if (error instanceof RpfmError && error.kind === "file_not_found") {
    console.log("No such table");
  }
}

await client.savePack(pack.key);
await client.disconnect();
```
```csharp
await using var client = await RpfmClient.ConnectAsync();
client.JobUpdated += status => Console.WriteLine($"{status["method"]}: {status["state"]}");

await client.SetGameAsync("warhammer_3");
var packKey = await client.OpenPackAsync("/path/to/my_mod.pack");

// Double the men of a unit.
var units = RpfmClient.PackFile(packKey, "db/land_units_tables/my_mod");
var page = await client.TableRowsAsync(units, [new { column = "key", op = "equals", value = "my_unit" }]);

foreach (var row in page!["rows"]!.AsArray())
{
    var columns = page["columns"]!.AsArray().Select(column => column!.GetValue<string>()).ToList();
    var numMen = row!["values"]![columns.IndexOf("num_men")]!.GetValue<int>();
    await client.EditTableAsync(packKey, "db/land_units_tables/my_mod", new { op = "update", index = row["index"]!.GetValue<int>(), values = new { num_men = numMen * 2 } });
}

try
{
    await client.TableRowsAsync(RpfmClient.PackFile(packKey, "db/missing_tables/data"));
}
catch (RpfmException error) when (error.Kind == "file_not_found")
{
    Console.WriteLine("No such table");
}

await client.SavePackAsync(packKey);
```
<!-- langtabs-end -->

## Adapting to Other Languages

The protocol is language-agnostic. To implement a client in another language:

1. **Connect** to `ws://127.0.0.1:45127/ws` using any WebSocket library, and wait for the `session.connected` notification.
2. **Send** JSON-RPC requests: `{ "jsonrpc": "2.0", "id": <number>, "method": "<method>", "params": { ... } }`.
3. **Receive** messages: the ones with an `id` are responses, matched to their request by it; the ones with a `method` are notifications.
4. **Handle errors** by their `data.kind`, not their message.
5. **Wait for jobs**: methods running as jobs return `{ "job": <id> }`. Call `job.wait` until the job's `state` is `finished`, `failed` or `cancelled`, or follow its `job.updated` notifications.
6. **Call** `session.disconnect` before closing the connection.

The Rust types of every method, in the `rpfm_ipc::api` module, are the reference for the JSON of params and results. Their docs are in `cargo doc -p rpfm_ipc --open`.
