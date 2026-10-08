//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Tests for mapping jobs to MCP tasks.

use serde_json::json;

use crate::jobs::JobRegistry;

use super::*;

#[test]
fn task_ids_round_trip_and_reject_other_ids() {
    assert_eq!(parse_task_id(&task_id(3, 7)), Some((3, 7)));
    assert_eq!(parse_task_id("7"), None);
    assert_eq!(parse_task_id("a-7"), None);
}

#[test]
fn jobs_map_to_tasks() {
    let registry = JobRegistry::default();
    let job = registry.create("session.set_game");
    let task = job_task(1, &registry.record(job).unwrap());
    assert_eq!(task.task.task_id, "1-1");
    assert_eq!(task.status(), TaskStatus::Working);
    assert_eq!(task.task.status_message.as_deref(), Some("Queued"));

    registry.start(job);
    registry.set_stage(job, "Loading");
    let task = job_task(1, &registry.record(job).unwrap());
    assert_eq!(task.status(), TaskStatus::Working);
    assert_eq!(task.task.status_message.as_deref(), Some("Loading"));

    registry.finish(job, RpcOutcome::Result(json!({"ok": true})));
    let task = job_task(1, &registry.record(job).unwrap());
    let TaskPayload::Completed { result } = task.payload else { panic!("finished jobs must complete their task") };
    assert_eq!(result["structuredContent"]["result"], json!({"ok": true}));
    assert_ne!(result.get("isError"), Some(&json!(true)));
}

#[test]
fn failed_jobs_complete_their_task_with_a_tool_error() {
    let registry = JobRegistry::default();
    let job = registry.create("session.set_game");
    registry.start(job);
    registry.finish(job, RpcOutcome::Error(ApiError::Internal("boom".to_owned()).into()));

    let task = job_task(1, &registry.record(job).unwrap());
    let TaskPayload::Completed { result } = task.payload else { panic!("failed jobs must complete their task") };
    assert_eq!(result["isError"], json!(true));
}

#[test]
fn cancelled_jobs_cancel_their_task() {
    let registry = JobRegistry::default();
    let job = registry.create("session.set_game");
    registry.cancel(job).unwrap();

    assert_eq!(job_task(1, &registry.record(job).unwrap()).status(), TaskStatus::Cancelled);
}
