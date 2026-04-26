//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! User feedback capture, posted to Sentry as a tagged event.
//!
//! The Sentry Rust SDK (v0.47) does not expose Sentry's dedicated User
//! Feedback envelope item, so feedback is sent as a regular `Level::Info`
//! event with a `rpfm.kind=user_feedback` tag and the user's text in
//! `extra.feedback`. The tag lets the [`crate::logger`] `before_send` hook
//! tell feedback apart from telemetry and crash reports, and lets you filter
//! for it in the Sentry UI.

use crate::actions::TELEMETRY_EVENT_TAG;
use crate::info;
use crate::logger::{sentry, Event, Level};

/// `rpfm.kind` tag value attached to user-feedback events.
pub(crate) const FEEDBACK_EVENT_VALUE: &str = "user_feedback";

/// Captures a single user-feedback message and ships it to Sentry.
///
/// The event is tagged so the logger's `before_send` filter lets it through
/// unconditionally. The user just clicked Send: that is consent in the
/// moment, regardless of the telemetry/crash-report toggles.
///
/// # Arguments
///
/// * `message` - The free-form text the user typed in the feedback dialog.
pub fn send_user_feedback(message: &str) {
    info!("Sending user feedback ({} chars)...", message.len());

    let mut event = Event::new();
    event.level = Level::Info;
    event.message = Some("User Feedback".to_string());
    event.tags.insert(TELEMETRY_EVENT_TAG.to_string(), FEEDBACK_EVENT_VALUE.to_string());
    event.extra.insert(
        "feedback".to_string(),
        sentry::protocol::Value::from(message.to_string()),
    );

    sentry::capture_event(event);
}
