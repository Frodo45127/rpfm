# rpfm_ipc

The API shared between `rpfm_server` and its clients.

This crate defines the type-safe API that clients (the Qt6 frontend, MCP clients and scripts) use to talk to the backend server, and the settings both sides share. Nothing here runs on its own.

> For user-facing project info (installation, building instructions, FAQ, contributing), see the [workspace README](../README.md) and the [manual][manual].
>
> This README targets developers consuming or working on the crate.

[manual]: https://frodo45127.github.io/rpfm/manual/

## Protocol

Clients call typed methods of the server over [JSON-RPC 2.0](https://www.jsonrpc.org/specification), sent as WebSocket text frames:

1. The client sends an `RpcRequest` with a unique `id`, a method name like `table.rows`, and its params.
2. The server answers with an `RpcResponse` with the same `id`, holding the result or an error with a typed `kind`.
3. Long methods run as jobs: they answer right away with a job ID, and report their progress and result in `job.updated` notifications.

Each method is a struct implementing the `Request` trait, which ties it to its method name and its response type, so the compiler checks that every call gets the right response:

```rust
pub trait Request: Serialize + DeserializeOwned {
    const METHOD: &'static str;
    type Response: Serialize + DeserializeOwned;
    const IS_JOB: bool = false;
}
```

The methods and the message format are documented in the [server docs][server_docs].

[server_docs]: https://frodo45127.github.io/rpfm/manual/server/ws-protocol.html

## Modules

- `api` — The JSON-RPC envelope (`RpcRequest`, `RpcResponse`, `RpcNotification`, `ApiError`), and the methods grouped by domain: `session`, `packs`, `files`, `tables`, `schema`, `diagnostics`, `search`, `references`, `notes`, `tools`, `translations`, `github`, `updates` and `jobs`.
- `helpers` — Data structures shared by several methods: `ContainerInfo`, `RFileInfo`, `VideoInfo`, `DependenciesInfo`, `DataSource`, `NewFile`, `APIResponse`, `SessionInfo`.
- `settings` — The settings store (`settings.json`) and the paths of the config folder. The UI owns the settings and sends them to its session; other sessions read them from disk.
- `settings_keys` — Typed string-key constants for every setting RPFM persists, so a typo becomes a compile error.

## Usage

This crate is meant to be consumed by `rpfm_ui` and `rpfm_server`, and by Rust tools that talk to a running `rpfm_server` over WebSocket.

```rust
use rpfm_ipc::api::RpcRequest;
use rpfm_ipc::api::packs::GetPackInfo;

let method = GetPackInfo { pack: pack_key };
let request = RpcRequest::new(1, &method)?;

// Serialize the request to JSON, send it over the WebSocket, and read the response with the same id...
let details = response.into_response::<GetPackInfo>()?;
```

## Related crates

- **rpfm_server** — Implements the server side of this protocol.
- **rpfm_ui** — Implements the client side.
- **rpfm_lib** — Provides the file types referenced in the params and responses.
- **rpfm_extensions** — Provides the higher-level workflows the methods expose.
- **rpfm_telemetry** — Owns the telemetry settings keys re-exported here.

## License

This project is licensed under the MIT License — see the [LICENSE](../LICENSE) file for details.

## Support

[![become_a_patron_button](https://user-images.githubusercontent.com/15714929/40394531-2130b9ce-5e24-11e8-91a2-bbf8e6e75d21.png)][Patreon]

[Patreon]: https://www.patreon.com/RPFM
