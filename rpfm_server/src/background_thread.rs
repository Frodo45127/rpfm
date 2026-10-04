//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Per-session request loop.
//!
//! Each [`Session`] spawns one thread running [`background_loop`]. The loop pulls [`SessionMessage`]s off the
//! session's mpsc channel and runs each request on the session's [`SessionState`] through [`crate::api`],
//! with the session's settings.
//!
//! Running requests serially per session is what keeps state consistent across many concurrent requests in the
//! same session: a `pack.save` followed by a `pack.close` always sees the right pack, even when the WebSocket
//! multiplexer is firing requests as fast as the client sends them. The exception are diagnostics checks: they stop
//! when other requests arrive, and get queued again behind them, so a check never makes the client wait.
//!
//! Telemetry: each request is recorded via [`rpfm_telemetry::record_action`] so usage counters reflect what the
//! session actually did.

use tokio::sync::mpsc::UnboundedReceiver;

use std::sync::Arc;

use rpfm_telemetry::info;

use crate::api::{self, JobContext};
use crate::session::{Session, SessionMessage};
use crate::state::SessionState;

/// The per-session request loop.
///
/// Receives requests from the session's mpsc `receiver` and runs them serially against the session's
/// [`SessionState`]. Requests get their response through their reply sender, and jobs through the session's jobs.
///
/// One instance runs per [`Session`], on a thread spawned by [`Session::new`]. The loop terminates when the session is
/// dropped or [`SessionMessage::Exit`] arrives.
pub fn background_loop(mut receiver: UnboundedReceiver<SessionMessage>, session: Arc<Session>) {
    let mut state = SessionState::new(session.clone());

    info!("Background Thread looping around…");
    while let Some(message) = receiver.blocking_recv() {
        match message {
            SessionMessage::Api(request, sender) => {
                session.dequeued();
                rpfm_telemetry::record_action(&request.method);
                let settings = session.settings();
                let context = JobContext::new(&|_| {}, &|_, _| true);
                let _ = sender.send(api::dispatch(&mut state, request, &settings, &context));
            }
            SessionMessage::Job(job, request) => {
                session.dequeued();
                let jobs = session.jobs().clone();
                if !jobs.start(job) {
                    continue;
                }

                rpfm_telemetry::record_action(&request.method);
                let settings = session.settings();
                let report_stage = |stage: &str| jobs.set_stage(job, stage);
                let report_progress = |done: usize, total: usize| {
                    jobs.set_progress(job, (done * 100 / total.max(1)).min(100) as u8);
                    !session.has_queued_requests()
                };

                let context = JobContext::new(&report_stage, &report_progress);

                // The request is kept to queue it again if the job stops for the requests behind it.
                let response = api::dispatch(&mut state, request.clone(), &settings, &context);
                if context.yielded() {
                    session.requeue_diagnostics_check(job, request);
                } else {
                    jobs.finish(job, response.outcome);
                }
            }
            SessionMessage::Autosave => {
                session.dequeued();
                state.autosave(&session.settings());
            }
            SessionMessage::Exit => break,
        }
    }
}
