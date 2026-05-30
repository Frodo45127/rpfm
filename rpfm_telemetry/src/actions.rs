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
//! session. On graceful shutdown the per-session counts are POSTed to
//! PostHog's `/batch/` capture endpoint as one event per distinct action,
//! with the invocation count carried as the `count` property so PostHog
//! insights can aggregate via `sum(properties.count)`.
//!
//! Telemetry is opt-out: the flag defaults to `true` so events captured
//! during early startup (before settings have been loaded) aren't lost.
//! Callers refresh it via [`set_usage_telemetry_enabled`] once they know
//! the user's preference. While disabled, [`track_action`] still emits a
//! log line for debugging, but no counters are kept.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{error, info, warn};

/// PostHog host all events are sent to (PostHog Cloud EU).
const DEFAULT_POSTHOG_HOST: &str = "https://eu.i.posthog.com";

/// Whether usage telemetry counter updates are enabled. On by default so
/// early-boot actions (before settings are loaded) are still counted;
/// callers may opt out via [`set_usage_telemetry_enabled`].
pub(crate) static USAGE_TELEMETRY_ENABLED: AtomicBool = AtomicBool::new(true);

/// Global action counter. Keys are action names, values are invocation counts.
static ACTION_COUNTS: LazyLock<RwLock<HashMap<String, u64>>> = LazyLock::new(|| RwLock::new(HashMap::new()));

/// PostHog project API key.
pub static POSTHOG_API_KEY: LazyLock<Arc<RwLock<String>>> = LazyLock::new(|| Arc::new(RwLock::new(String::new())));

/// Identifier sent as `distinct_id` on every PostHog event.
static DISTINCT_ID: LazyLock<RwLock<String>> = LazyLock::new(|| {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    RwLock::new(format!("rpfm-ephemeral-{}-{}", std::process::id(), nanos))
});

/// Extra properties attached to every PostHog event in the flush (release,
/// os, is_beta, ...). Callers populate this via [`set_event_property`]
/// before the flush runs.
static EVENT_PROPERTIES: LazyLock<RwLock<HashMap<String, serde_json::Value>>> = LazyLock::new(|| RwLock::new(HashMap::new()));

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

/// Sets the PostHog project API key used by [`flush`].
///
/// An empty string disables the PostHog flush entirely.
pub fn set_posthog_api_key(key: &str) {
    if let Ok(mut guard) = POSTHOG_API_KEY.write() {
        *guard = key.to_string();
    }
}

/// Sets the `distinct_id` attached to every PostHog event in the flush.
pub fn set_distinct_id(id: &str) {
    if let Ok(mut guard) = DISTINCT_ID.write() {
        *guard = id.to_string();
    }
}

/// Attaches a property to every event sent in the next [`flush`].
pub fn set_event_property(key: &str, value: serde_json::Value) {
    if let Ok(mut guard) = EVENT_PROPERTIES.write() {
        guard.insert(key.to_string(), value);
    }
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

/// Sends accumulated action telemetry to PostHog and clears the counters.
///
/// No-op when telemetry is disabled, no counters are pending, or no
/// PostHog API key has been configured.
pub fn flush(source: &str) {
    let counts = match ACTION_COUNTS.write() {
        Ok(mut guard) => guard.drain().collect::<HashMap<String, u64>>(),
        Err(_) => return,
    };

    if counts.is_empty() {
        return;
    }

    let distinct_id = match DISTINCT_ID.read() {
        Ok(guard) => guard.clone(),
        Err(_) => return,
    };

    let shared_props = EVENT_PROPERTIES.read().map(|g| g.clone()).unwrap_or_default();

    let source = source.to_string();
    let batch = counts.into_iter().map(|(action, count)| {
        let mut props = shared_props.clone();
        props.insert("distinct_id".to_string(), serde_json::Value::from(distinct_id.clone()));
        props.insert("count".to_string(), serde_json::Value::from(count));
        props.insert("source".to_string(), serde_json::Value::from(source.clone()));
        serde_json::json!({ "event": action, "properties": props })
    }).collect::<Vec<_>>();

    info!("Flushing action telemetry to PostHog ({} distinct actions)...", batch.len());
    post_events(batch, "action telemetry");
}

/// Captures a single arbitrary PostHog event immediately.
///
/// Mainly for the user feedback dialog.
///
/// # Arguments
///
/// * `event` - The PostHog event name.
/// * `properties` - Event-specific properties, merged over the shared ones.
pub fn capture_event(event: &str, properties: HashMap<String, serde_json::Value>) {
    let distinct_id = match DISTINCT_ID.read() {
        Ok(guard) => guard.clone(),
        Err(_) => return,
    };

    let mut props = EVENT_PROPERTIES.read().map(|g| g.clone()).unwrap_or_default();
    props.insert("distinct_id".to_string(), serde_json::Value::from(distinct_id));
    for (key, value) in properties {
        props.insert(key, value);
    }

    let batch = vec![serde_json::json!({ "event": event, "properties": props })];

    info!("Capturing PostHog event `{}`...", event);
    post_events(batch, "event capture");
}

/// POSTs a batch of pre-built PostHog events to the `/batch/` capture endpoint.
///
/// The HTTP POST runs on a freshly spawned OS thread because some callers
/// (the server's shutdown path) invoke this from inside a tokio runtime, and
/// `reqwest::blocking` panics if used on a tokio worker thread.
///
/// # Arguments
///
/// * `batch` - The PostHog event objects to send (each `{event, properties}`).
/// * `context` - Short label used in the skip/success/failure log lines.
fn post_events(batch: Vec<serde_json::Value>, context: &str) {
    let api_key = match POSTHOG_API_KEY.read() {
        Ok(guard) => guard.clone(),
        Err(_) => return,
    };

    if api_key.is_empty() {
        info!("Skipping {context} send: PostHog API key not configured.");
        return;
    }

    let payload = serde_json::json!({
        "api_key": api_key,
        "batch": batch,
    });

    let url = format!("{}/batch/", DEFAULT_POSTHOG_HOST);
    let context = context.to_string();

    let handle = std::thread::spawn(move || -> Result<(), String> {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .map_err(|err| format!("building client: {err}"))?;

        let response = client.post(&url)
            .json(&payload)
            .send()
            .map_err(|err| format!("posting batch: {err}"))?;

        if !response.status().is_success() {
            return Err(format!("PostHog responded {}: {}", response.status(), response.text().unwrap_or_default()));
        }

        Ok(())
    });

    match handle.join() {
        Ok(Ok(())) => info!("PostHog {context} sent."),
        Ok(Err(err)) => warn!("PostHog {context} send failed: {err}"),
        Err(_) => error!("PostHog {context} send thread panicked."),
    }
}
