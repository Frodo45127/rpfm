# rpfm_telemetry

`rpfm_telemetry` bundles the three observability concerns that share RPFM's Sentry lifecycle:

-   **Structured logging** — re-exports the `log` crate's `debug!`/`info!`/`warn!`/`error!`/`trace!` macros so every crate can emit log lines without pulling in Sentry.
-   **Crash reporting** — `Logger::init` installs panic hooks, writes local crash reports, and wires Sentry for release builds (breadcrumbs, sessions, automatic panic capture).
-   **Action telemetry** — a lightweight counter that aggregates anonymous usage data and ships it to Sentry on graceful shutdown.

Libraries (`rpfm_lib`, `rpfm_extensions`, `rpfm_ui_common`, …) depend directly on the plain `log` crate and stay Sentry-free. The executables (`rpfm_ui`, `rpfm_server`) depend on this crate to wire up the full stack.

## Initialization

The logger must be initialized at program startup. The returned `ClientInitGuard` keeps Sentry alive and must remain in scope for the process lifetime.

```rust
use rpfm_telemetry::{Logger, SENTRY_DSN, info, release_name};

*SENTRY_DSN.write().unwrap() = "https://<key>@sentry.io/<project>".to_owned();

let _guard = Logger::init(
    &std::path::Path::new("crash_reports"),
    true,                 // verbose
    true,                 // set global logger
    release_name!(),      // release identifier
).expect("Failed to initialize logging");

info!("Application started");
```

In release builds panics are automatically captured, dumped locally as TOML, and uploaded to Sentry. In debug builds the Sentry guard is created with an empty DSN, so crash reports stay local.

## Action telemetry

Call `track_action` at the top of user-facing event handlers. The function emits an `info!` log line and, when usage telemetry is enabled, increments a counter.

```rust
use rpfm_telemetry::{track_action, record_action, flush};

// Full trace + count.
track_action("Open PackFile");

// Count only, for callers that already log the action elsewhere.
record_action("OpenPackFiles");

// On graceful shutdown, flush the aggregated counts to Sentry.
// The label lets UI and server events be told apart.
flush("UI Action Telemetry");
```

## The two toggles

Telemetry is **opt-out** — both toggles default to `true`. Users can disable either independently via the preferences dialog.

| Toggle                      | Setting key              | What it controls                                                                       |
|-----------------------------|--------------------------|----------------------------------------------------------------------------------------|
| `set_usage_telemetry_enabled` | `enable_usage_telemetry` | Whether `track_action`/`record_action` update counters and whether `flush()` is sent.  |
| `set_crash_reports_enabled`   | `enable_crash_reports`   | Whether panic reports, auto-captured errors and session tracking reach Sentry.         |

Both settings are plumbed through `rpfm_ipc::settings_keys`. Executables read them at startup and whenever the user toggles them in the Settings dialog, then call the matching `set_*_enabled` function.

Internally, each toggle maps to an `AtomicBool` consulted from a `before_send` filter in `Logger::init`. Usage-telemetry events are tagged with `rpfm.kind = usage_telemetry` so the filter can route them to the usage toggle; everything else (panics, breadcrumbs, sessions, manually captured events) is governed by the crash-reports toggle. Runtime changes take effect immediately.

## Manually sending diagnostic payloads

For explicit user-initiated uploads (e.g. sharing a schema patch) the crate exposes helpers on `Logger`:

```rust
Logger::send_event(&guard, Level::Info, "Schema updated", None)?;
Logger::upload_patches(&guard, "warhammer_3", &patches)?;
Logger::upload_definitions(&guard, "warhammer_3", &definitions)?;
```

These paths are gated only on the Sentry client being enabled (release build + non-empty DSN), not on the user-facing toggles, because they represent explicit user actions to share data.

## Related crates

-   **rpfm_lib** — Core library for file-format handling (pure `log` dependency).
-   **rpfm_extensions** — Higher-level modding features (pure `log` dependency).
-   **rpfm_ipc** — IPC protocol and shared settings keys (including the telemetry toggles).
-   **rpfm_ui** — Qt6 desktop frontend.
-   **rpfm_server** — WebSocket/MCP backend.

## License

This project is licensed under the MIT License — see the [LICENSE](../LICENSE) file for details.

## Support

[![become_a_patron_button](https://user-images.githubusercontent.com/15714929/40394531-2130b9ce-5e24-11e8-91a2-bbf8e6e75d21.png)][Patreon]

[Patreon]: https://www.patreon.com/RPFM
