# rpfm_ui

The Qt6 desktop frontend for ***Rusted PackFile Manager***.

`rpfm_ui` is the user-facing application most people interact with. It ships the main window, menus, editors, tools and dialogs that let users open, inspect, edit and save PackFiles for supported Total War games. Heavy lifting (file I/O, schema handling, dependency resolution, diagnostics, etc.) is delegated to `rpfm_server` over a local WebSocket, keeping the UI thread responsive and letting multiple frontends share a backend.

> For user-facing project info (installation, building instructions, FAQ, contributing), see the [workspace README](../README.md) and the [manual][manual].
>
> This README targets developers working on the UI crate itself.

[manual]: https://frodo45127.github.io/rpfm/

## Architecture

```
┌──────────────────────────┐    WebSocket (IPC)    ┌──────────────────────────┐
│        rpfm_ui           │ ────────────────────▶ │      rpfm_server         │
│  (Qt6 desktop app)       │ ◀──────────────────── │  (pack/schema/fs work)   │
└──────────────────────────┘                       └──────────────────────────┘
         │                                                  │
         │  depends on                                      │  depends on
         ▼                                                  ▼
   rpfm_ui_common  ·  rpfm_ipc  ·  rpfm_telemetry  ·  rpfm_lib  ·  rpfm_extensions
```

- **Qt6 bindings** come from the pinned `qt_*` crates under `3rdparty/src/ritual/`. All Qt interaction happens on the main thread.
- **Server spawning** is automatic: `main.rs` probes `127.0.0.1:45127`, and if the server isn't running, builds (debug) or launches (release) `rpfm_server`. In debug mode this means a `cargo build -p rpfm_server` before launch.
- **Command dispatch** goes through `crate::communications::send_ipc_command(...)` / `send_ipc_command_async(...)`, correlating requests to responses via message IDs defined in `rpfm_ipc::messages`.
- **Settings** live on the server and are read through a thread-local cache in `settings_ui::backend`; writes go to the server and invalidate the cache.
- **Logging, crash reporting and action telemetry** are all routed through `rpfm_telemetry`. `track_action("…")` at the top of user-facing slot closures feeds anonymous usage counters that flush to Sentry on exit.

## Module layout

Top-level modules in `src/`:

- `app_ui/` — main window, menu bar, top-level slots, tab bar for open files.
- `command_palette_ui/` — Ctrl+P style fuzzy command/file palette.
- `communications/` — WebSocket client, request/response correlation, background async helpers.
- `dependencies_ui/` — Parent-mods and vanilla dependencies tree panel.
- `diagnostics_ui/` — Diagnostics panel and ignore rules.
- `ffi/` — C++ FFI bridges (KTextEditor, KIconThemes, shortcut dialog, custom models, optional model renderer).
- `global_search_ui/` — Global search and replace panel.
- `mymod_ui/` — "MyMod" project dialog.
- `pack_tree/` — Shared tree-model operations used by PackFile and dependencies views.
- `packfile_contents_ui/` — PackFile contents tree, context menus, add/extract/rename/etc.
- `packedfile_views/` — Editors for each file type: `table`, `text`, `animpack`, `audio`, `video`, `bmd`, `vmd`, `rigidmodel`, `portrait_settings`, `unit_variant`, `image`, `esf`, `decoder`, `notes`, `packfile_settings`, `uic`, `anim_fragment_battle`, `anims_table`, `matched_combat`, `group_formations`, `dependencies_manager`, `external`.
- `references_ui/` — Find-references results panel.
- `session_ui/` — Server-session picker/manager.
- `settings_ui/` — Preferences dialog plus backend settings cache.
- `tools/` — Integrated tools: `faction_painter`, `unit_editor`, `translator` (gated on `enable_tools`).
- `ui/`, `ui_state/` — Top-level UI construction and cross-cutting UI state.
- `updater_ui/` — Update dialog (program, schemas, TW autogen, old AK).
- `views/` — Reusable view components (table, debug, filter, search).
- `welcome_page_ui/` — Start page shown when no PackFile is open.

Each UI area usually follows the same pattern:

- `mod.rs` — widget construction, public API.
- `slots.rs` — one struct of Qt slot closures, one `::new(...)` constructor.
- `connections.rs` — wires signals to slots.

## Feature flags

- `enable_tools` *(default)* — compiles the integrated tools module (faction painter, unit editor, translator).
- `support_model_renderer` — enables the C++ model renderer behind rigidmodel/BMD previews. Requires the renderer library on the link path.
- `support_uic` — propagates UIC support to `rpfm_lib` and `rpfm_ui_common`.
- `strict_subclasses_compilation` — flips some FFI subclass generation on for stricter builds.
- `only_for_the_brave` — exposes experimental features in the About dialog.

## Building & running

Release build:

```bash
cargo build --release -p rpfm_ui
```

Debug build (auto-builds `rpfm_server` on first launch):

```bash
cargo run -p rpfm_ui
```

System requirements (development):

- **Qt6** (with headers).
- **KDE Frameworks**: KTextEditor, KIconThemes, KColorScheme (used through FFI for the text editor and themed icons).
- **CMake + a C++17 toolchain** — needed by the `qt_*` ritual bindings and some FFI bridges.

On Windows, KDE binaries must be reachable at runtime; on Linux the `rpfm-bin` AUR package / flatpak handle this automatically.

See the [compilation instructions][build] in the manual for platform-specific notes.

[build]: https://frodo45127.github.io/rpfm/chapter_comp.html

## Telemetry

`rpfm_ui` uses `rpfm_telemetry` to:

- Capture panics as local crash reports and (in release builds) upload them to Sentry.
- Maintain anonymous action counters that flush to Sentry on graceful shutdown as `"UI Action Telemetry"` events.

Both are **opt-out** and independently controllable from **Preferences → Telemetry**:

- **Enable Usage Telemetry** — gates action counters.
- **Enable Crash Reports** — gates panic reports, auto-captured errors and session tracking.

See `rpfm_telemetry`'s README for the full model.

## Related crates

- **rpfm_lib** — Core file-format handling.
- **rpfm_extensions** — Dependencies, diagnostics, global search, optimizer, translator.
- **rpfm_ui_common** — Shared Qt6 utilities (`clone!` macro, pointer helpers, locale, icons).
- **rpfm_ipc** — Command/response protocol + settings keys.
- **rpfm_telemetry** — Logging, crash reporting, action telemetry.
- **rpfm_server** — Backend doing the actual PackFile/schema/filesystem work.

## License

This project is licensed under the MIT License — see the [LICENSE](../LICENSE) file for details.

## Support

[![become_a_patron_button](https://user-images.githubusercontent.com/15714929/40394531-2130b9ce-5e24-11e8-91a2-bbf8e6e75d21.png)][Patreon]

[Patreon]: https://www.patreon.com/RPFM
