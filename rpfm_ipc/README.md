# rpfm_ipc

The IPC protocol shared between `rpfm_ui` and `rpfm_server`.

This crate defines the type-safe message contract that the Qt6 frontend and the backend server use to talk to each other. Both sides depend on it; nothing here runs on its own.

> For user-facing project info (installation, building instructions, FAQ, contributing), see the [workspace README](../README.md) and the [manual][manual].
>
> This README targets developers consuming or working on the crate.

[manual]: https://frodo45127.github.io/rpfm/manual/

## Protocol

The frontend and the server speak JSON over a local WebSocket. Each payload is wrapped in a `Message<T>`:

```rust
pub struct Message<T: Debug> {
    pub id: u64,
    pub data: T,
}
```

The `id` correlates a response to the request that produced it, which lets the UI keep many requests in flight simultaneously without blocking. Outgoing messages carry `Command`; incoming ones carry `Response`.

## Modules

- `messages` — Core protocol: `Message<T>` wrapper, `Command` enum, `Response` enum, plus shared enums like `OperationalMode`.
- `helpers` — Marshalling types: `ContainerInfo`, `RFileInfo`, `VideoInfo`, `DependenciesInfo`, `DataSource`, `NewFile`, `APIResponse`, `SessionInfo`.
- `settings_keys` — Typed string-key constants for every setting RPFM persists, plus `SettingsSnapshot` for batch transfers.

## Commands

`Command` is a large enum (~150 variants) covering everything the UI can ask the server to do. Variants are grouped roughly into:

- **Pack lifecycle.** New, open, save, save-as, close, type changes, compression, optimization, "load all CA packs", SiegeAI patching.
- **PackedFile operations.** Create, add, decode, extract, rename, delete, copy/cut/paste, duplicate, AnimPack-specific operations.
- **Dependencies.** Get table lists, resolve table versions, merge files, update dependency cache.
- **Search & navigation.** Global search and replace, find-references, go-to-definition, loc-key lookups.
- **Schema.** Load, save, query definitions, apply patches.
- **Settings.** Per-type get/set (`bool`, `i32`, `f32`, `String`, `PathBuf`, `Vec<String>`) plus a batch `SettingsGetAll` that returns a `SettingsSnapshot`.
- **Updates.** Check/apply updates for the program, schemas, Lua autogen, old AK files, translations.
- **MyMod.** Operational-mode get/set, install/uninstall, import/export.
- **Sessions.** Client disconnect, autosave, backup, exit.
- **Advanced.** Cascade edition, map packing, startpos build pre/post/cleanup, animation ID updates, diagnostics, TSV import/export, external-program integration.

## Responses

`Response` is a similarly broad enum (~80 variants). Names encode the payload type so the UI can pattern-match without runtime checks:

- Simple — `Success`, `Error(String)`, `Bool(bool)`, `I32(i32)`, `String(String)`, `PathBuf(PathBuf)`.
- File-typed — `DBRFileInfo`, `LocRFileInfo`, `TextRFileInfo`, `AnimFragmentBattleRFileInfo`, …
- Aggregate — `GlobalSearchVecRFileInfo`, `VecContainerPathVecRFileInfo`, `HashMapDataSourceHashMapStringRFile`.
- Settings — `SettingsAll(SettingsSnapshot)`.
- Session — `SessionConnected(u64)` is sent unsolicited right after a WebSocket handshake so the client learns its session ID.

## settings_keys

`settings_keys` exposes every setting the server reads or writes as a `pub const &str`. Both sides import these constants instead of stringly-typing keys, so a typo becomes a compile error. Categories include:

- Paths (`MYMOD_BASE_PATH`, `SECONDARY_PATH`, …).
- General (default game, language, autosave cadence, font, recent files).
- Editor behaviour (`LAZY_LOADING`, `DISABLE_UUID_REGENERATION`, `EXPAND_TREEVIEW_WHEN_ADDING_ITEMS`, …).
- Tables (`TIGHT_TABLE_MODE`, `ENABLE_LOOKUPS`, `ENABLE_ICONS`, `HIDE_UNUSED_COLUMNS`, …).
- Diagnostics (`ENABLE_DEBUG_MENU`, `DIAGNOSTICS_TRIGGER_*`, …).
- Telemetry (`ENABLE_USAGE_TELEMETRY`, `ENABLE_CRASH_REPORTS`).
- AI/external services (`AI_API_URL`, `AI_API_KEY`, `AI_MODEL`, `DEEPL_API_KEY`).
- Optimizer toggles (one constant per step).
- Theme colour keys for light and dark variants.

`SettingsSnapshot` bundles the whole settings store into per-type maps so the UI can pull everything in one round trip on startup, then keep a local cache.

## Helper types

Types in `helpers` are the "view models" the server hands back when it doesn't want to ship a full decoded file:

- `ContainerInfo` — Pack name, path, version, type, compression, timestamp.
- `RFileInfo` — File path, container name, timestamp, file type.
- `VideoInfo` — Format, codec, dimensions, frame count, framerate.
- `DependenciesInfo` — Paths grouped by source.
- `DataSource` — `PackFile`, `GameFiles`, `ParentFiles`, `AssKitFiles`, `ExternalFile`.
- `NewFile` — Construction parameters for new files (AnimPack, DB, Loc, PortraitSettings, Text, VMD, WSModel).
- `APIResponse` — Update-check outcome.
- `SessionInfo` — Per-session snapshot used by the `/sessions` endpoint and the session picker.

## Usage

This crate is meant to be consumed by `rpfm_ui` and `rpfm_server` only. Direct use elsewhere should be limited to building tools that talk to a running `rpfm_server` over WebSocket.

```rust
use rpfm_ipc::messages::{Command, Message};

let request = Message {
    id: 1,
    data: Command::OpenPackFiles(vec![path]),
};
// serialize to JSON, send over the WebSocket, await the matching response by id...
```

## Related crates

- **rpfm_server** — Implements the server side of this protocol.
- **rpfm_ui** — Implements the client side.
- **rpfm_lib** — Provides the file types referenced in `Command`/`Response` payloads.
- **rpfm_extensions** — Provides the higher-level workflows the protocol exposes.
- **rpfm_telemetry** — Owns the telemetry settings keys re-exported here.

## License

This project is licensed under the MIT License — see the [LICENSE](../LICENSE) file for details.

## Support

[![become_a_patron_button](https://user-images.githubusercontent.com/15714929/40394531-2130b9ce-5e24-11e8-91a2-bbf8e6e75d21.png)][Patreon]

[Patreon]: https://www.patreon.com/RPFM
