# rpfm_ui_common

Shared Qt6 utilities for RPFM's UI code.

This crate holds the pieces that any Qt-based RPFM frontend needs — the main `rpfm_ui` application, integrated tools, and any future Qt frontend. It stays intentionally thin: no PackFile logic, no UI layouts, just cross-cutting helpers.

## What's inside

### `clone!` macro

A macro for cloning variables into closures — indispensable for Qt slot handlers where closures need to capture by value without fighting the borrow checker.

```rust
use rpfm_ui_common::clone;

let slot = SlotNoArgs::new(&parent, clone!(
    app_ui,
    pack_file_contents_ui => move || {
        do_something(&app_ui, &pack_file_contents_ui);
    }
));
```

Supports closures with any number of arguments and an optional `mut` prefix per captured variable.

### `utils`

Qt ↔ Rust plumbing helpers:

- `atomic_from_cpp_box` / `atomic_from_q_box` / `atomic_from_ptr` — stash Qt pointers behind `AtomicPtr` so they can live in statics.
- `q_ptr_from_atomic` / `ptr_from_atomic` / `ref_from_atomic` — reverse operations.
- `show_dialog` — standard informational/error dialog with title and body.
- `find_widget` — look up a child widget by `objectName` and downcast it.
- `load_template` — load a `.ui` file into a parented widget.
- `create_grid_layout`, `clear_layout` — layout helpers.
- `log_to_status_bar_2` — write a timestamped line to a Qt status bar.

### `locale`

Fluent-based (`.ftl`) locale system. Loads `locale/English_en.ftl` as a fallback and overlays a user-selected locale on top. `tr("key")`, `qtr("key")`, and `qtre("key", &[args])` look up translated strings, with the last returning a `QString` ready for Qt APIs.

### `icons`

Helpers for loading SVG icons bundled with the application (reads from `ASSETS_PATH`).

### `tools`

Scaffolding for RPFM's integrated tools (shared base dialog, message-widget helpers, common result-reporting patterns).

### Static paths & identity

Set once at startup and then read from anywhere:

- `PROGRAM_PATH` / `ASSETS_PATH` — resolve differently in debug vs. release (cwd vs. exe-relative vs. Linux/flatpak paths).
- `ORG_DOMAIN` / `ORG_NAME` / `APP_NAME` — mirror `QCoreApplication`'s identity, used by `QSettings` and the locale subsystem.
- `FULL_DATE_FORMAT` / `SLASH_DMY_DATE_FORMAT` / `SLASH_MDY_DATE_FORMAT` — preparsed `time` formatters for common RPFM date layouts.

### Tree-model constants

A handful of `i32` role constants (`ROOT_NODE_TYPE`, `ITEM_PACK_KEY`, …) used by RPFM's PackFile tree views so consumers agree on the same item-role meanings.

## Feature flags

- `support_uic` — enables loading widgets from compiled Qt `.uic` files (needed by some `rpfm_lib` integrations with the same flag).

## Requirements

- Qt6 (via the `qt_*` crates under `3rdparty/src/ritual/`).

Most public functions touching Qt are `unsafe` because the underlying Qt bindings are. Follow standard Qt rules: create and touch widgets only from the main thread, and keep parent widgets alive for the lifetime of their children.

## Related crates

- **rpfm_ui** — Qt6 desktop frontend; the main consumer of this crate.
- **rpfm_lib** — Core file-format library.
- **rpfm_telemetry** — Logging, crash reporting and action telemetry.
- **rpfm_ipc** — IPC protocol shared with `rpfm_server`.

## License

This project is licensed under the MIT License — see the [LICENSE](../LICENSE) file for details.
