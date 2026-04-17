//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Anonymous action telemetry, shared between the UI and the server.
//!
//! Tracks which actions (UI slots, server commands, ...) are used during a
//! session and sends aggregated counts to Sentry on graceful shutdown. This
//! helps understand which features are most used and guides development
//! priorities.
//!
//! Telemetry is opt-out: the flag defaults to `true` so events captured
//! during early startup (before settings have been loaded) aren't lost.
//! Callers refresh it via [`set_usage_telemetry_enabled`] once they know
//! the user's preference. While disabled, [`track_action`] still emits a
//! log line for debugging, but no counters are kept.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{LazyLock, RwLock};

use crate::info;
use crate::logger::{sentry, Event, Level};

/// Tag key attached to usage-telemetry Sentry events so the [`crate::logger`]
/// `before_send` hook can tell them apart from crash reports.
pub(crate) const TELEMETRY_EVENT_TAG: &str = "rpfm.kind";
pub(crate) const TELEMETRY_EVENT_VALUE: &str = "usage_telemetry";

/// Whether usage telemetry counter updates are enabled. On by default so
/// early-boot actions (before settings are loaded) are still counted;
/// callers may opt out via [`set_usage_telemetry_enabled`].
pub(crate) static USAGE_TELEMETRY_ENABLED: AtomicBool = AtomicBool::new(true);

/// Global action counter. Keys are action names, values are invocation counts.
static ACTION_COUNTS: LazyLock<RwLock<HashMap<String, u64>>> = LazyLock::new(|| RwLock::new(HashMap::new()));

/// Enables or disables usage-telemetry counter updates.
///
/// Callers should set this once at startup (from the `enable_usage_telemetry`
/// setting) and refresh it whenever the setting changes. Pending counts
/// accumulated while enabled are preserved and will still be flushed.
pub fn set_usage_telemetry_enabled(enabled: bool) {
    USAGE_TELEMETRY_ENABLED.store(enabled, Ordering::Relaxed);
}

/// Returns the current usage-telemetry enabled state.
pub fn is_usage_telemetry_enabled() -> bool {
    USAGE_TELEMETRY_ENABLED.load(Ordering::Relaxed)
}

/// Records a single action occurrence.
///
/// Also emits an `info!` log line for consistency with the existing logging.
/// If telemetry is disabled, only the log line is emitted.
pub fn track_action(action: &str) {
    info!("Triggering `{}` By Slot", action);
    record_action(action);
}

/// Records a single action occurrence without emitting a log line.
///
/// Useful for callers (like the server) that already log incoming actions
/// elsewhere and don't want a duplicate log line. No-op when telemetry is
/// disabled.
pub fn record_action(action: &str) {
    if USAGE_TELEMETRY_ENABLED.load(Ordering::Relaxed) {
        if let Ok(mut counts) = ACTION_COUNTS.write() {
            *counts.entry(action.to_string()).or_insert(0) += 1;
        }
    }
}

/// Sends accumulated action telemetry to Sentry and clears the counters.
///
/// Should be called once on graceful shutdown, while the Sentry guard is
/// still alive. `source` is used as the Sentry event message (e.g.
/// `"UI Action Telemetry"` or `"Server Action Telemetry"`), allowing UI and
/// server events to be distinguished in Sentry.
///
/// The event is tagged as usage-telemetry so the logger's `before_send`
/// filter lets it through even when crash reports are disabled.
pub fn flush(source: &str) {
    let counts = match ACTION_COUNTS.write() {
        Ok(mut guard) => guard.drain().collect::<HashMap<String, u64>>(),
        Err(_) => return,
    };

    if counts.is_empty() {
        return;
    }

    info!("Flushing action telemetry ({} distinct actions)...", counts.len());

    let mut event = Event::new();
    event.level = Level::Info;
    event.message = Some(source.to_string());
    event.tags.insert(TELEMETRY_EVENT_TAG.to_string(), TELEMETRY_EVENT_VALUE.to_string());

    for (action, count) in &counts {
        event.extra.insert(
            action.clone(),
            sentry::protocol::Value::from(*count),
        );
    }

    sentry::capture_event(event);
}
