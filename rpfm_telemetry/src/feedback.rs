//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! User feedback capture, posted to PostHog as a dedicated event.
//!
//! Feedback is sent as a PostHog `user_feedback` event with the user's text
//! carried in the `feedback` property, reusing the PostHog plumbing in
//! [`crate::actions`] (API key, host, `distinct_id` and shared event
//! properties). Sending is not gated on the usage-telemetry toggle: the user
//! just clicked Send, which is consent in the moment regardless of the
//! telemetry/crash-report settings.

use std::collections::HashMap;

use crate::actions::capture_event;
use crate::info;

/// PostHog event name used for user-feedback submissions.
const FEEDBACK_EVENT_NAME: &str = "user_feedback";

/// Captures a single user-feedback message and ships it to PostHog.
///
/// # Arguments
///
/// * `message` - The free-form text the user typed in the feedback dialog.
pub fn send_user_feedback(message: &str) {
    info!("Sending user feedback ({} chars)...", message.len());

    let mut properties = HashMap::new();
    properties.insert("feedback".to_string(), serde_json::Value::from(message.to_string()));

    capture_event(FEEDBACK_EVENT_NAME, properties);
}
