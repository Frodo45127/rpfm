# rpfm_ipc

`rpfm_ipc` is a crate that defines the Inter-Process Communication (IPC) protocol used between the RPFM frontend (`rpfm_ui`) and the backend server (`rpfm_server`). It provides the data structures for commands and responses, ensuring type-safe communication between the two processes.

## Protocol Structure

The communication protocol is built around three main components:

-   `Message<T>`: A generic wrapper for all messages sent between the frontend and the server. It contains a unique `id` to correlate requests with their corresponding responses, and a `data` field payload.

    ```rust
    pub struct Message<T: Debug> {
        pub id: u64,
        pub data: T,
    }
    ```

-   `Command`: An enum that defines all the possible commands the frontend can send to the server. Each variant represents a specific action to be performed, such as opening a PackFile, saving a file, or running a diagnostic check.

-   `Response`: An enum that defines all the possible responses the server can send back to the frontend. Each variant corresponds to the result of a specific command, carrying data on success or an error message on failure.

## Usage

This crate is not intended to be used as a standalone application. It is a dependency of `rpfm_server` and `rpfm_ui`, providing the shared language they need to communicate.

When the frontend needs to perform an action, it serializes a `Message<Command>` into JSON and sends it over a WebSocket connection to the server. The server then deserializes the message, executes the command, and sends back a `Message<Response>` containing the result. The unique `id` in the message wrapper allows the frontend to match the response to the correct original request.
