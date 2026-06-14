# Agent Guidelines for RPFM

This document provides guidelines for contributing to the Rusted PackFile Manager (RPFM) codebase. These rules MUST be followed by all AI coding agents and contributors.

## Project Context

RPFM is a Rust + Qt6 desktop application for opening, inspecting, editing and saving PackFiles for every Total War since *Empire: Total War*.

Workspace layout:

### Libraries

- `rpfm_lib` — Core file-format library: packs, schemas, DB, Loc, RigidModel, audio, video, etc.
- `rpfm_extensions` — Higher-level workflows: dependencies, diagnostics, search, optimizer, translator, glTF export.
- `rpfm_ipc` — Command/response protocol shared between UI and server.
- `rpfm_telemetry` — Logging, crash reporting and opt-out action telemetry (Sentry-backed).
- `rpfm_ui_common` — Shared Qt6 helpers used by every UI consumer.

### Executables

- `rpfm_ui` — The Qt6 desktop application most users run. Depends on Qt6 bindings under `3rdparty/`.
- `rpfm_server` — Headless backend doing the heavy file/schema/filesystem work. Exposes WebSocket + MCP endpoints. The UI spawns it automatically.

The `3rdparty/` directory is excluded from the workspace. It contains vendored `ritual`-generated Qt6 bindings (`qt_core`, `qt_gui`, `qt_widgets`, `qt_ui_tools`, `cpp_core`). **Do not touch files in `3rdparty/` (except `3rdparty/src/qt_rpfm_extensions`) unless explicitly asked.**

MSRV is `1.85`, edition `2021`.

## Your Core Principles

Code you write MUST be clear, correct, and appropriately optimized for the task:

- Maximize algorithmic big-O efficiency for memory and runtime where it matters (PackFile decoding/encoding, diagnostics, search).
- Use `rayon` for CPU-bound parallelism when iterating over large collections (tables, files within a pack).
- Follow idiomatic Rust and maximize code reuse (DRY).
- No extra code beyond what is necessary to solve the problem. No speculative abstractions.

If you believe code is not ready to hand off, do another pass before stopping.

## Preferred Tools and Crates

The workspace already pins the canonical set of dependencies in the top-level `Cargo.toml`. When adding functionality, prefer what's already there before introducing new crates:

- Building / dependencies: `cargo` via the workspace `Cargo.toml`.
- Error handling: `thiserror` for library error types, `anyhow` for application-level errors / glue code.
- Serialization: `serde` with `serde_json`, `ron` (schemas and settings), `bitcode` (compact binary encoding), `toml` where applicable.
- Logging: the `log` crate (funneled through `rpfm_telemetry`). Use `log::error!`, `log::warn!`, `log::info!`, `log::debug!`, `log::trace!` instead of `println!` / `eprintln!` / `dbg!`.
- Async runtime: `tokio` (used mainly by `rpfm_server` and `rpfm_ipc`).
- Parallelism: `rayon` for CPU-bound work; `crossbeam` channels for threaded message passing.
- Accessors: `getset` for generated getters/setters on structs with many private fields (matches existing style).
- HTTP: `reqwest` (the `blocking` feature is used in parts of the codebase; match the surrounding module's style).
- Self-update: `self_update` (already wired up, do not reinvent).
- Qt6 GUI: `qt_core`, `qt_gui`, `qt_widgets`, `qt_ui_tools` via the vendored `ritual` bindings. Qt interop goes through `cpp_core` (`CppBox`, `Ptr`, `MutPtr`).
- MCP (server only): `rmcp` with `schemars` for tool schemas.

## Code Style and Formatting

- **NEVER** run `rustfmt` / `cargo fmt`. The maintainer has explicitly rejected it — past experiments grew the codebase from ~18k to >30k lines of noise. Match the surrounding code's manual formatting instead.
- Clippy-suggested fixes are welcome. Run `cargo clippy` and apply reasonable suggestions.
- **MUST** use 4 spaces for indentation (never tabs).
- **MUST** use meaningful, descriptive variable and function names.
- Use `snake_case` for functions/variables/modules, `PascalCase` for types/traits, `SCREAMING_SNAKE_CASE` for constants.
- Follow Rust API Guidelines and idiomatic Rust conventions.
- **NEVER** use emoji, or unicode that emulates emoji (e.g. check/cross glyphs). The only exception is when tests need to exercise multibyte characters.

## Comments and "No Black Magic"

> **HARD RULE — NO COMMENTS LONGER THAN 2 LINES INSIDE FUNCTIONS.** No exceptions. If a longer explanation is genuinely needed, lift it into the function's doc comment instead (doc comments may be longer). This is the most-violated rule here — re-read it before writing any in-body comment.

Per `CONTRIBUTING.md`: **explain what your code does** and **no black magic code**. The maintainer uses this project to learn Rust, so unexplained cleverness is a regression.

- When using a non-obvious Rust feature (lifetime tricks, complex trait bounds, `unsafe`, manual `Drop`, custom `Deref`), add a short comment explaining *why*.
- Comments should describe intent and rationale, not restate what the code obviously does.
- Minimize comments inside function bodies and don't overexplain: add one only where the code isn't self-evident.
- Keep comments up to date with code changes. Delete stale ones.

## Documentation

- **MUST** include doc comments for public functions, structs, enums, methods, and modules in library crates (`rpfm_lib`, `rpfm_extensions`, `rpfm_ipc`, `rpfm_telemetry`, `rpfm_ui_common`).
- **MUST** document function parameters, return values, and errors when non-trivial.
- Match the existing doc-comment style in the crate you're editing.
- **MUST NOT** write over-detailed function descriptions. The canonical shape is: one short line summarising intent, then `# Arguments` and `# Returns` (and `# Errors` when applicable), each with a one-line description per item. Don't explain internal logic, don't justify implementation choices in prose, and don't enumerate call sites by name (those belong in the PR description and rot as the codebase evolves). See `rpfm_extensions/src/optimizer/mod.rs::optimize` for a model.

## Type System

- **MUST** leverage Rust's type system to prevent bugs at compile time.
- **NEVER** use `.unwrap()` in library code. Use `.expect("...")` only for invariant violations with a descriptive message.
- **MUST** use meaningful custom error types with `thiserror` in library crates.
- Prefer `Option<T>` over sentinel values.
- Use newtypes to distinguish semantically different values of the same underlying type.

## Error Handling

- **NEVER** use `.unwrap()` in production code paths.
- **MUST** use `Result<T, E>` for fallible operations.
- Use `thiserror` for library-level error enums; use `anyhow` for application-level glue (binaries, high-level orchestration).
- **MUST** propagate errors with the `?` operator where appropriate.
- Provide meaningful error messages with context (`.context(...)` from `anyhow`, or typed variants with `thiserror`).

## Function Design

- **MUST** keep functions focused on a single responsibility.
- **MUST** prefer borrowing (`&T`, `&mut T`) over ownership when possible.
- Limit function parameters to 5 or fewer; use a config struct for more.
- Return early to reduce nesting.
- Use iterators and combinators over explicit loops where clearer.

## Struct and Enum Design

- **MUST** keep types focused on a single responsibility.
- **MUST** derive common traits: `Debug`, `Clone`, `PartialEq` where appropriate.
- Use `#[derive(Default)]` when a sensible default exists.
- Prefer composition over inheritance-like patterns.
- Make fields private and expose them via `getset` (matching the rest of the codebase) or explicit accessors.

## Testing

- **MUST** write unit tests for new parsing/encoding logic in `rpfm_lib` and new algorithms in `rpfm_extensions`.
- Use the built-in `#[test]` attribute and `cargo test`.
- Place tests in `#[cfg(test)]` modules or the conventional `tests/` directory.
- Follow the Arrange-Act-Assert pattern.
- Do not commit commented-out tests.
- Note: CI builds and tests only the library crates plus `rpfm_server`. `rpfm_ui` is not built in CI (Qt6 is unavailable there), so UI changes must be verified locally.

## Imports and Dependencies

- **MUST** avoid wildcard imports (`use module::*`) except for preludes, test modules (`use super::*`), and prelude re-exports.
- **MUST** avoid long inline path chains like `crate::xxxx::yyyyy::method(...)` at call sites. Add a `use crate::xxxx::yyyyy;` and call `yyyyy::method(...)` instead.
- Add new dependencies to the workspace `[workspace.dependencies]` table first, then reference them from per-crate `Cargo.toml` files. Match the existing version-pinning style (`"^X"` or exact pin).
- Organize imports: standard library, external crates, local modules. Match the surrounding file.

## Rust Best Practices

- **NEVER** use `unsafe` unless absolutely necessary; when used, document the safety invariants in a comment above the block. Qt FFI via `cpp_core` is the main legitimate source of `unsafe` in this codebase — follow the patterns already in `rpfm_ui` and `rpfm_ui_common`.
- **MUST** call `.clone()` explicitly on non-`Copy` types; avoid hidden clones in closures and iterators.
- **MUST** use pattern matching exhaustively; avoid catch-all `_` patterns when listing variants keeps the compiler honest about future additions.
- **MUST** use the `format!` macro for string formatting.
- Use iterators and iterator adapters over manual loops.
- Use `.enumerate()` instead of manual counter variables.
- Prefer `if let` and `while let` for single-pattern matching.

## Memory and Performance

- **MUST** avoid unnecessary allocations; prefer `&str` over `String` when possible.
- **MUST** use `Cow<'_, str>` when ownership is conditionally needed.
- Use `Vec::with_capacity()` when the size is known (common when decoding tables with a known row count).
- Prefer borrowing; use `Arc` and `Rc` only when shared ownership is actually required.
- Large files are commonly processed in parallel with `rayon`; follow the patterns in `rpfm_extensions::diagnostics` and similar modules.

## Concurrency

- **MUST** use `Send` and `Sync` bounds appropriately.
- Use `tokio` for async work (primarily `rpfm_server` and `rpfm_ipc`).
- Use `rayon` for CPU-bound parallelism.
- Use `crossbeam` channels for threaded message passing.
- Avoid `Mutex` when `RwLock` or lock-free alternatives are appropriate.

## Qt6 / UI Notes (`rpfm_ui`, `rpfm_ui_common`)

- Interop goes through `cpp_core::{CppBox, Ptr, MutPtr}`; most Qt methods are `unsafe`. Scope `unsafe` blocks tightly and mirror the existing conventions.
- Slots and signals follow the `qt_core` patterns already used in these crates — match the surrounding style rather than inventing new ones.
- KDE Frameworks (`KTextEditor`, `KIconThemes`, `KColorScheme`) are consumed through hand-written bindings or Qt's plugin loader; don't add new FFI lightly.
- `rpfm_ui` changes are not exercised by CI. Build and smoke-test locally before claiming done.

## Security

- **NEVER** store secrets, API keys, or crash-reporter DSNs hardcoded in a way that leaks into version control. Keep them in `.env` or project-level untracked config.
- Ensure `.env` stays in `.gitignore`.
- **NEVER** log user-identifying or path-leaking information through the telemetry pipeline beyond what `rpfm_telemetry` already sanitizes.

## Version Control

- **MUST** write clear, descriptive commit messages. Match the concise, imperative style of recent commits (`git log` for examples).
- **NEVER** commit commented-out code; delete it.
- **NEVER** commit debug `println!` / `eprintln!` / `dbg!` statements.
- **NEVER** commit credentials or sensitive data.
- The repository uses a beta convention where patch numbers `>= 99` mark a beta release (e.g. `4.7.106` is beta, `4.7.0` is stable); no `-beta` suffix is used.

## Tools

- **MUST** use `clippy` for linting and apply reasonable suggestions: `cargo clippy --workspace -- -D warnings`.
- **MUST** ensure code compiles with no new warnings.
- **NEVER** run `rustfmt` / `cargo fmt` on this repository.
- Use `cargo test` for running tests.
- Use `cargo doc` for generating documentation.

## Before Committing

- [ ] All tests pass (`cargo test` on the affected crates).
- [ ] No new compiler warnings (`cargo check`).
- [ ] Clippy is clean or only shows pre-existing issues (`cargo clippy --workspace -- -D warnings`).
- [ ] **Did NOT run `cargo fmt`**.
- [ ] If `rpfm_ui` code was touched, build and smoke-test it locally (CI does not build the UI).
- [ ] Public library items have doc comments.
- [ ] No commented-out code, `println!` debugging, or `dbg!` macros left behind.
- [ ] No hardcoded credentials.

---

**Remember:** Prioritize clarity and maintainability over cleverness. When in doubt, add a comment explaining why — the maintainer (and future you) will thank you.
