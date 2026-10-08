//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Tests for the lifecycle of jobs: their states, cancellation, pruning and waiting.

use serde_json::json;

use super::*;

#[test]
fn jobs_go_through_their_states() {
    let registry = JobRegistry::default();
    let job = registry.create("session.set_game");
    assert_eq!(registry.status(job).unwrap().state, JobState::Queued);

    assert!(registry.start(job));
    registry.set_stage(job, "Loading");
    assert_eq!(registry.status(job).unwrap().state, JobState::Running { stage: Some("Loading".to_owned()), progress: None });

    let mut updates = registry.subscribe();
    registry.set_progress(job, 40);
    registry.set_progress(job, 40);
    registry.set_progress(job, 30);
    assert_eq!(registry.status(job).unwrap().state, JobState::Running { stage: Some("Loading".to_owned()), progress: Some(40) });
    assert!(updates.try_recv().is_ok());
    assert!(updates.try_recv().is_err(), "unchanged or lower progress must not be notified");

    registry.finish(job, RpcOutcome::Result(json!({"ok": true})));
    assert_eq!(registry.status(job).unwrap().state, JobState::Finished { result: json!({"ok": true}) });
}

#[test]
fn only_queued_jobs_can_be_cancelled() {
    let registry = JobRegistry::default();
    let queued = registry.create("a");
    let running = registry.create("b");
    assert!(registry.start(running));

    assert_eq!(registry.cancel(queued).unwrap().state, JobState::Cancelled);
    assert!(!registry.start(queued), "cancelled jobs must not start");
    assert!(matches!(registry.cancel(running), Err(ApiError::InvalidParams(_))));
    assert_eq!(registry.cancel(999), Err(ApiError::JobNotFound(999)));
}

#[test]
fn running_jobs_can_be_requeued_or_cancelled() {
    let registry = JobRegistry::default();
    let job = registry.create("diagnostics.run");
    assert!(!registry.requeue(job), "queued jobs can't be requeued");

    assert!(registry.start(job));
    assert!(registry.requeue(job));
    assert_eq!(registry.status(job).unwrap().state, JobState::Queued);

    assert!(registry.start(job));
    registry.finish_cancelled(job);
    assert_eq!(registry.status(job).unwrap().state, JobState::Cancelled);
}

#[test]
fn old_ended_jobs_are_forgotten() {
    let registry = JobRegistry::default();
    let first = registry.create("a");
    registry.cancel(first).unwrap();

    for _ in 0..MAX_ENDED_JOBS {
        let job = registry.create("a");
        registry.cancel(job).unwrap();
    }

    // Pruning happens when jobs are created, so one more is needed to push the first one out.
    registry.create("a");
    assert!(registry.status(first).is_none());
}

#[tokio::test]
async fn wait_returns_when_the_job_ends_or_times_out() {
    let registry = std::sync::Arc::new(JobRegistry::default());
    let job = registry.create("a");

    let timed_out = registry.wait(job, Duration::from_millis(20)).await.unwrap();
    assert_eq!(timed_out.state, JobState::Queued);

    let waiter = {
        let registry = registry.clone();
        tokio::spawn(async move { registry.wait(job, Duration::from_secs(10)).await })
    };

    registry.start(job);
    registry.finish(job, RpcOutcome::Result(json!(1)));
    assert_eq!(waiter.await.unwrap().unwrap().state, JobState::Finished { result: json!(1) });
}
