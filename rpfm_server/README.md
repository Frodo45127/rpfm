# rpfm_server

`rpfm_server` is the backend server for RPFM (Rusted PackFile Manager). It provides a WebSocket and MCP (Model Context Protocol) interface for the frontend to interact with the filesystem and perform PackFile operations.

## Compilation

To compile the server, you can use the following commands:

**For a debug build:**
```bash
cargo build -p rpfm_server
```

**For a release build:**
```bash
cargo build --release --bin rpfm_server
```

## Usage

The server is usually spawned automatically by the `rpfm_gpui` frontend. However, you can also run it manually.

When started, the server will listen on `127.0.0.1:45127` by default. It exposes two endpoints:

-   `/ws`: A WebSocket endpoint for general commands and responses. The frontend connects to this to send commands and receive results. It supports session management, allowing a client to disconnect and reconnect to the same session.
-   `/mcp`: An HTTP endpoint for the Model Context Protocol, used for more specialized, streamable operations.

To run the server, execute the compiled binary:

```bash
./target/release/rpfm_server
```

The server will log its output to the standard error stream.
