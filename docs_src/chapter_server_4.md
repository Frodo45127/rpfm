# Client Example

This page provides a complete client implementation example in TypeScript. The same patterns apply to any language with WebSocket and JSON support.

## TypeScript Client

The following class wraps the WebSocket connection and provides typed convenience methods for common operations.

### Connection and Session Handling

<!-- langtabs-start -->
```typescript
class RpfmClient {
  private ws: WebSocket;
  private nextId = 1;
  private pending = new Map<number, {
    resolve: (resp: any) => void;
    reject: (err: Error) => void;
  }>();
  public sessionId: number | null = null;

  constructor(url = "ws://127.0.0.1:45127/ws") {
    this.ws = new WebSocket(url);
    this.ws.onmessage = (event) => {
      const msg = JSON.parse(event.data);

      // Handle SessionConnected (unsolicited, id=0)
      if (typeof msg.data === "object" && "SessionConnected" in msg.data) {
        this.sessionId = msg.data.SessionConnected;
        console.log(`Connected to session ${this.sessionId}`);
        return;
      }

      const handler = this.pending.get(msg.id);
      if (handler) {
        this.pending.delete(msg.id);
        if (typeof msg.data === "object" && "Error" in msg.data) {
          handler.reject(new Error(msg.data.Error));
        } else {
          handler.resolve(msg.data);
        }
      }
    };
  }

  send(command: object | string): Promise<any> {
    return new Promise((resolve, reject) => {
      const id = this.nextId++;
      this.pending.set(id, { resolve, reject });
      this.ws.send(JSON.stringify({ id, data: command }));
    });
  }
```
```csharp
using System.Collections.Concurrent;
using System.Net.WebSockets;
using System.Text;
using System.Text.Json;

public class RpfmClient : IDisposable
{
    private readonly ClientWebSocket _ws = new();
    private int _nextId = 1;
    private readonly ConcurrentDictionary<int, TaskCompletionSource<JsonElement>> _pending = new();
    public int? SessionId { get; private set; }

    public async Task ConnectAsync(string url = "ws://127.0.0.1:45127/ws")
    {
        await _ws.ConnectAsync(new Uri(url), CancellationToken.None);
        _ = Task.Run(ReceiveLoop);
    }

    private async Task ReceiveLoop()
    {
        var buffer = new byte[65536];
        while (_ws.State == WebSocketState.Open)
        {
            var result = await _ws.ReceiveAsync(buffer, CancellationToken.None);
            var json = Encoding.UTF8.GetString(buffer, 0, result.Count);
            var msg = JsonDocument.Parse(json).RootElement;

            var data = msg.GetProperty("data");

            // Handle SessionConnected (unsolicited, id=0)
            if (data.ValueKind == JsonValueKind.Object
                && data.TryGetProperty("SessionConnected", out var sid))
            {
                SessionId = sid.GetInt32();
                Console.WriteLine($"Connected to session {SessionId}");
                continue;
            }

            var id = msg.GetProperty("id").GetInt32();
            if (_pending.TryRemove(id, out var tcs))
            {
                if (data.ValueKind == JsonValueKind.Object
                    && data.TryGetProperty("Error", out var err))
                {
                    tcs.SetException(new Exception(err.GetString()));
                }
                else
                {
                    tcs.SetResult(data);
                }
            }
        }
    }

    public async Task<JsonElement> SendAsync(object command)
    {
        var id = Interlocked.Increment(ref _nextId);
        var tcs = new TaskCompletionSource<JsonElement>();
        _pending[id] = tcs;

        var msg = JsonSerializer.Serialize(new { id, data = command });
        var bytes = Encoding.UTF8.GetBytes(msg);
        await _ws.SendAsync(bytes, WebSocketMessageType.Text, true, CancellationToken.None);

        return await tcs.Task;
    }
```
<!-- langtabs-end -->

Key points:
- Each request gets a unique `id` for correlation
- The `SessionConnected` message arrives immediately on connection with `id: 0`
- Error responses are automatically converted to rejected promises
- Multiple requests can be in flight simultaneously

### Typed Convenience Methods

<!-- langtabs-start -->
```typescript
  // --- Pack management ---

  async openPack(paths: string[]): Promise<[string, any]> {
    const resp = await this.send({ OpenPackFiles: paths });
    return resp.StringContainerInfo;
  }

  async listOpenPacks(): Promise<[string, any][]> {
    const resp = await this.send("ListOpenPacks");
    return resp.VecStringContainerInfo;
  }

  async closePack(packKey: string): Promise<void> {
    await this.send({ ClosePack: packKey });
  }

  async closeAllPacks(): Promise<void> {
    await this.send("CloseAllPacks");
  }

  async savePack(packKey: string): Promise<any> {
    const resp = await this.send({ SavePack: packKey });
    return resp.ContainerInfo;
  }

  async savePackAs(packKey: string, path: string): Promise<any> {
    const resp = await this.send({ SavePackAs: [packKey, path] });
    return resp.ContainerInfo;
  }

  // --- File operations ---

  async getTreeView(packKey: string): Promise<[any, any[]]> {
    const resp = await this.send({ GetPackFileDataForTreeView: packKey });
    return resp.ContainerInfoVecRFileInfo;
  }

  async decodeFile(packKey: string, path: string, source = "PackFile"): Promise<any> {
    return this.send({ DecodePackedFile: [packKey, path, source] });
  }

  async deleteFiles(packKey: string, paths: any[]): Promise<any[]> {
    const resp = await this.send({ DeletePackedFiles: [packKey, paths] });
    return resp.VecContainerPath;
  }

  async extractFiles(
    packKey: string,
    paths: Record<string, any[]>,
    destPath: string,
    asTsv = false,
  ): Promise<[string, string[]]> {
    const resp = await this.send({
      ExtractPackedFiles: [packKey, paths, destPath, asTsv]
    });
    return resp.StringVecPathBuf;
  }

  // --- Game selection ---

  async setGame(gameKey: string, rebuildDeps: boolean): Promise<void> {
    await this.send({ SetGameSelected: [gameKey, rebuildDeps] });
  }

  // --- Settings ---

  async getSetting(key: string): Promise<string> {
    const resp = await this.send({ SettingsGetString: key });
    return resp.String;
  }

  async getAllSettings(): Promise<{
    bools: Record<string, boolean>;
    ints: Record<string, number>;
    floats: Record<string, number>;
    strings: Record<string, string>;
  }> {
    const resp = await this.send("SettingsGetAll");
    const [bools, ints, floats, strings] = resp.SettingsAll;
    return { bools, ints, floats, strings };
  }

  // --- Lifecycle ---

  async disconnect(): Promise<void> {
    await this.send("ClientDisconnecting");
    this.ws.close();
  }
}
```
```csharp
    // --- Pack management ---

    public async Task<JsonElement> OpenPackAsync(string[] paths)
    {
        var resp = await SendAsync(new { OpenPackFiles = paths });
        return resp.GetProperty("StringContainerInfo");
    }

    public async Task<JsonElement> ListOpenPacksAsync()
    {
        var resp = await SendAsync("ListOpenPacks");
        return resp.GetProperty("VecStringContainerInfo");
    }

    public async Task ClosePackAsync(string packKey)
    {
        await SendAsync(new { ClosePack = packKey });
    }

    public async Task CloseAllPacksAsync()
    {
        await SendAsync("CloseAllPacks");
    }

    public async Task<JsonElement> SavePackAsync(string packKey)
    {
        var resp = await SendAsync(new { SavePack = packKey });
        return resp.GetProperty("ContainerInfo");
    }

    public async Task<JsonElement> SavePackAsAsync(string packKey, string path)
    {
        var resp = await SendAsync(new { SavePackAs = new[] { packKey, path } });
        return resp.GetProperty("ContainerInfo");
    }

    // --- File operations ---

    public async Task<JsonElement> GetTreeViewAsync(string packKey)
    {
        var resp = await SendAsync(new { GetPackFileDataForTreeView = packKey });
        return resp.GetProperty("ContainerInfoVecRFileInfo");
    }

    public async Task<JsonElement> DecodeFileAsync(
        string packKey, string path, string source = "PackFile")
    {
        return await SendAsync(new { DecodePackedFile = new[] { packKey, path, source } });
    }

    public async Task<JsonElement> DeleteFilesAsync(string packKey, object[] paths)
    {
        var resp = await SendAsync(new { DeletePackedFiles = new object[] { packKey, paths } });
        return resp.GetProperty("VecContainerPath");
    }

    public async Task<JsonElement> ExtractFilesAsync(
        string packKey,
        Dictionary<string, object[]> paths,
        string destPath,
        bool asTsv = false)
    {
        var resp = await SendAsync(
            new { ExtractPackedFiles = new object[] { packKey, paths, destPath, asTsv } });
        return resp.GetProperty("StringVecPathBuf");
    }

    // --- Game selection ---

    public async Task SetGameAsync(string gameKey, bool rebuildDeps)
    {
        await SendAsync(new { SetGameSelected = new object[] { gameKey, rebuildDeps } });
    }

    // --- Settings ---

    public async Task<string> GetSettingAsync(string key)
    {
        var resp = await SendAsync(new { SettingsGetString = key });
        return resp.GetProperty("String").GetString()!;
    }

    public async Task<JsonElement> GetAllSettingsAsync()
    {
        var resp = await SendAsync("SettingsGetAll");
        return resp.GetProperty("SettingsAll");
    }

    // --- Lifecycle ---

    public async Task DisconnectAsync()
    {
        await SendAsync("ClientDisconnecting");
        await _ws.CloseAsync(
            WebSocketCloseStatus.NormalClosure, "done", CancellationToken.None);
    }

    public void Dispose() => _ws.Dispose();
}
```
<!-- langtabs-end -->

### Usage Example

<!-- langtabs-start -->
```typescript
async function main() {
  const client = new RpfmClient();

  // Wait for connection
  await new Promise<void>((resolve) => {
    client["ws"].onopen = () => resolve();
  });
  console.log(`Session ID: ${client.sessionId}`);

  // Select a game
  await client.setGame("warhammer_3", true);

  // Open a pack file
  const [packKey, containerInfo] = await client.openPack([
    "/home/user/mods/my_mod.pack"
  ]);
  console.log(`Opened pack: ${containerInfo.file_name} (key: ${packKey})`);

  // Get the file tree
  const [info, files] = await client.getTreeView(packKey);
  console.log(`Pack contains ${files.length} files`);

  // Decode a DB table
  const decoded = await client.decodeFile(
    packKey,
    "db/units_tables/data",
    "PackFile"
  );
  if ("DBRFileInfo" in decoded) {
    const [db, fileInfo] = decoded.DBRFileInfo;
    console.log(`Table: ${db.table.table_name}, rows: ${db.table.table_data.length}`);
  }

  // Extract files to disk
  const [extractPath, extractedFiles] = await client.extractFiles(
    packKey,
    { PackFile: [{ File: "db/units_tables/data" }] },
    "/tmp/extracted"
  );
  console.log(`Extracted ${extractedFiles.length} files to ${extractPath}`);

  // Save and disconnect
  await client.savePack(packKey);
  await client.disconnect();
}

main().catch(console.error);
```
```csharp
using var client = new RpfmClient();
await client.ConnectAsync();

// Wait briefly for SessionConnected
await Task.Delay(500);
Console.WriteLine($"Session ID: {client.SessionId}");

// Select a game
await client.SetGameAsync("warhammer_3", true);

// Open a pack file
var packResult = await client.OpenPackAsync(new[] { "/home/user/mods/my_mod.pack" });
var packKey = packResult[0].GetString()!;
var containerInfo = packResult[1];
Console.WriteLine($"Opened pack: {containerInfo.GetProperty("file_name")} (key: {packKey})");

// Get the file tree
var treeView = await client.GetTreeViewAsync(packKey);
var files = treeView[1];
Console.WriteLine($"Pack contains {files.GetArrayLength()} files");

// Decode a DB table
var decoded = await client.DecodeFileAsync(packKey, "db/units_tables/data", "PackFile");
if (decoded.TryGetProperty("DBRFileInfo", out var dbResult))
{
    var db = dbResult[0];
    var table = db.GetProperty("table");
    Console.WriteLine(
        $"Table: {table.GetProperty("table_name")}, " +
        $"rows: {table.GetProperty("table_data").GetArrayLength()}");
}

// Extract files to disk
var extractResult = await client.ExtractFilesAsync(
    packKey,
    new Dictionary<string, object[]>
    {
        ["PackFile"] = new object[] { new { File = "db/units_tables/data" } }
    },
    "/tmp/extracted");
var extractPath = extractResult[0].GetString();
var extractedFiles = extractResult[1];
Console.WriteLine($"Extracted {extractedFiles.GetArrayLength()} files to {extractPath}");

// Save and disconnect
await client.SavePackAsync(packKey);
await client.DisconnectAsync();
```
<!-- langtabs-end -->

## Adapting to Other Languages

The protocol is language-agnostic. To implement a client in another language:

1. **Connect** to `ws://127.0.0.1:45127/ws` using any WebSocket library
2. **Send** JSON messages in the format `{ "id": <number>, "data": <command> }`
3. **Receive** JSON messages and match responses by `id`
4. **Handle** the `SessionConnected` message (id=0) on connect
5. **Send** `ClientDisconnecting` before closing the connection

The JSON serialization follows Rust's serde conventions:
- Unit variants: `"VariantName"`
- Newtype variants: `{ "VariantName": value }`
- Tuple variants: `{ "VariantName": [v1, v2, ...] }`
