# rpfm_log

Crash reporting and structured logging with Sentry integration for RPFM.

This crate provides comprehensive error tracking and logging capabilities for the Rusted PackFile Manager (RPFM) project. It is inspired by the `human-panic` crate but offers more configurability and integration with Sentry for production error tracking.

## Features

### Local Crash Reports

When a panic occurs, a detailed crash report is saved locally as a TOML file containing:

- Program name and version
- Build type (debug/release)
- Operating system information
- Panic message and location
- Full backtrace

### Sentry Integration

In release builds, crashes and events are automatically uploaded to Sentry for:

- Centralized error tracking
- Session health monitoring
- Breadcrumb trails
- Custom event uploads with attachments (e.g., schema patches and definitions)

### Runtime Logging

Standard logging macros are re-exported and available throughout the application:

- `info!` - Informational messages (verbose mode only)
- `warn!` - Warning messages
- `error!` - Error messages

Logging output is sent to both the terminal (via `simplelog`) and Sentry breadcrumbs.

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
rpfm_log = "4.7"
```

### Initialization

The logger must be initialized at program startup:

```rust
use rpfm_log::{Logger, info, warn};
use std::path::Path;

let _guard = Logger::init(
    Path::new("logs/crash_reports"),
    true,  // verbose mode
    true,  // set global logger
    Some("rpfm@5.0.0".into()),
)?;

info!("Application started");
warn!("Something might be wrong");

// Keep _guard alive until program exit
```

The returned `ClientInitGuard` must be kept alive for the duration of the program. Dropping it shuts down Sentry and flushes pending events.

## Related Crates

- **rpfm_lib** - Core library for file format handling
- **rpfm_extensions** - Higher-level features (dependencies, diagnostics, search, optimizer)
- **rpfm_ui** - Qt-based desktop application
- **rpfm_server** - WebSocket/MCP backend server

## License

This project is licensed under the MIT License - see the [LICENSE](../LICENSE) file for details.

## Support

[![become_a_patron_button](https://user-images.githubusercontent.com/15714929/40394531-2130b9ce-5e24-11e8-91a2-bbf8e6e75d21.png)][Patreon]

[Patreon]: https://www.patreon.com/RPFM
