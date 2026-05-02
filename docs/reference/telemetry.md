# Telemetry & crash reports

RPFM ships two pieces of opt-out telemetry, both controlled from **Preferences → Telemetry**:

- **Enable Usage Telemetry** — anonymous counters of which actions get used.
- **Enable Crash Reports** — automatic upload of panic reports and breadcrumbs to Sentry when RPFM crashes.

Both default to **on**. Both take effect immediately.

## What gets sent

### Usage telemetry

When enabled, RPFM increments per-action counters in memory and flushes them to Sentry as a single event when the program shuts down gracefully (or, on the server side, when a session disconnects). The events are tagged so server and UI activity can be told apart.

Each counter is just `(action_name, count)`. There's no payload, no path, no Pack name, no game, no user identifier — just "Open PackFile happened 7 times in this session".

Examples of action names that actually appear in the codebase: `Open PackFile Menu`, `Open PackedFile Full`, `Save Pack`, `Diagnostics Check`, `Cascade Edition`, `Optimize PackFile`, `Open In External Program`, `Merge Tables`. They're whatever string a `rpfm_telemetry::track_action(...)` call passes — there's no central registry.

### Crash reports

When enabled, RPFM:

- Captures **panics** as they happen.
- Writes a local crash report under the `error/` folder.
- Uploads the crash report to Sentry, along with breadcrumbs (a structured trail of recent operations in the session).
- Tracks **session start / session end** so we can tell how often crashes occur per session.

Crash reports include the crashing thread's stack trace, RPFM's version, the OS family, and the breadcrumb trail. They do **not** include the contents of any open Pack, file paths, or anything else identifying.

In **debug builds** the Sentry guard is created with an empty DSN, so panics dump locally but don't upload anywhere.

## Local crash reports

Even with crash report uploads disabled, RPFM still writes panic reports locally under `<config>/error/` as TOML files. They're useful for filing bug reports yourself: when something crashes, look in `error/`, attach the file to a [GitHub issue](https://github.com/Frodo45127/rpfm/issues), and that gives the developers exactly what an automatic upload would have.

## API surface

If you're embedding `rpfm_telemetry` into your own RPFM-derived tool, the [`rpfm_telemetry` API docs](../../api/rpfm_telemetry/index.html) cover the full surface. The README in the crate has more on the design.
