//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! State of the jobs of a session. See [`rpfm_ipc::api::jobs`].
//!
//! Jobs run on the session's background thread like any other request. The registry only keeps their
//! state, so it can be checked, waited for and cancelled from outside the background thread, and
//! broadcasts every change so connected clients get notified.

use serde::Serialize;
use serde_json::Value;
use tokio::sync::{broadcast, mpsc::UnboundedSender};
use tokio::time::{timeout_at, Duration, Instant};

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::SystemTime;

use rpfm_ipc::api::{ApiError, Done, Request, RpcOutcome, RpcRequest, RpcResponse};
use rpfm_ipc::api::jobs::{CancelJob, DEFAULT_WAIT_SECS, GetJobStatus, JobState, JobStatus, WaitForJob};

use crate::api::parse_params;

/// Amount of ended jobs whose state is kept. Older ones are forgotten.
const MAX_ENDED_JOBS: usize = 100;

/// Capacity of the channel broadcasting job changes. Slow listeners skip the changes they miss.
const UPDATES_CAPACITY: usize = 64;

/// State of the jobs of a session.
#[derive(Debug)]
pub struct JobRegistry {

    /// ID of the next job.
    next_id: AtomicU64,

    /// State of each job, by ID.
    jobs: Mutex<BTreeMap<u64, JobRecord>>,

    /// Channel every job change is sent to.
    updates: broadcast::Sender<JobStatus>,
}

/// State of a job, with when it was created and when it last changed.
#[derive(Clone, Debug)]
pub struct JobRecord {

    /// State of the job.
    pub status: JobStatus,

    /// When the job was created.
    pub created_at: SystemTime,

    /// When the state of the job last changed.
    pub updated_at: SystemTime,
}

impl Default for JobRegistry {
    fn default() -> Self {
        Self {
            next_id: AtomicU64::new(1),
            jobs: Mutex::new(BTreeMap::new()),
            updates: broadcast::channel(UPDATES_CAPACITY).0,
        }
    }
}

impl JobRegistry {

    /// Returns if any job is queued or running.
    pub fn has_unfinished_jobs(&self) -> bool {
        self.jobs.lock().unwrap().values().any(|record| !record.status.state.has_ended())
    }

    /// Registers a new queued job.
    ///
    /// # Returns
    ///
    /// The ID of the job.
    pub fn create(&self, method: &str) -> u64 {
        let job = self.next_id.fetch_add(1, Ordering::SeqCst);
        self.set(JobStatus { job, method: method.to_owned(), state: JobState::Queued });
        job
    }

    /// Marks a queued job as running.
    ///
    /// # Returns
    ///
    /// `false` if the job was cancelled or is unknown, so it must not run.
    pub fn start(&self, job: u64) -> bool {
        self.transition(job, |state| matches!(state, JobState::Queued).then_some(JobState::Running { stage: None, progress: None })).is_some()
    }

    /// Sets the step a running job is on.
    pub fn set_stage(&self, job: u64, stage: &str) {
        self.transition(job, |state| matches!(state, JobState::Running { .. }).then(|| JobState::Running { stage: Some(stage.to_owned()), progress: None }));
    }

    /// Sets how far along a running job is, from 0 to 100.
    ///
    /// Only increases are kept, as parallel steps of a job may report their progress out of order.
    pub fn set_progress(&self, job: u64, progress: u8) {
        self.transition(job, |state| match state {
            JobState::Running { stage, progress: current } if current.is_none_or(|current| current < progress) => Some(JobState::Running { stage: stage.clone(), progress: Some(progress) }),
            _ => None,
        });
    }

    /// Moves a running job back to the queue.
    ///
    /// # Returns
    ///
    /// `false` if the job isn't running.
    pub fn requeue(&self, job: u64) -> bool {
        self.transition(job, |state| matches!(state, JobState::Running { .. }).then_some(JobState::Queued)).is_some()
    }

    /// Ends a job as cancelled, even if it already started.
    pub fn finish_cancelled(&self, job: u64) {
        self.transition(job, |state| (!state.has_ended()).then_some(JobState::Cancelled));
    }

    /// Ends a job with the outcome of its method.
    pub fn finish(&self, job: u64, outcome: RpcOutcome) {
        let state = match outcome {
            RpcOutcome::Result(result) => JobState::Finished { result },
            RpcOutcome::Error(error) => JobState::Failed { error },
        };

        self.transition(job, |_| Some(state));
    }

    /// Cancels a job that hasn't started yet.
    ///
    /// # Errors
    ///
    /// Fails if the job is unknown, or if it already started.
    pub fn cancel(&self, job: u64) -> Result<JobStatus, ApiError> {
        let already = |state: &JobState| if state.has_ended() { "ended" } else { "started" };
        match self.status(job) {
            None => Err(ApiError::JobNotFound(job)),
            Some(status) => self.transition(job, |state| matches!(state, JobState::Queued).then_some(JobState::Cancelled))
                .ok_or_else(|| ApiError::InvalidParams(format!("The job {job} already {}, so it can't be cancelled.", already(&status.state)))),
        }
    }

    /// Returns the state of a job, or `None` if it's unknown.
    pub fn status(&self, job: u64) -> Option<JobStatus> {
        self.record(job).map(|record| record.status)
    }

    /// Returns the state of a job with when it was created and last changed, or `None` if it's unknown.
    pub fn record(&self, job: u64) -> Option<JobRecord> {
        self.jobs.lock().unwrap().get(&job).cloned()
    }

    /// Returns a receiver of every job change from now on.
    pub fn subscribe(&self) -> broadcast::Receiver<JobStatus> {
        self.updates.subscribe()
    }

    /// Waits for a job to end, up to `timeout`.
    ///
    /// # Returns
    ///
    /// The state of the job when it ended, or when the wait timed out. `None` if the job is unknown.
    pub async fn wait(&self, job: u64, timeout: Duration) -> Option<JobStatus> {

        // Subscribe before checking the state, so an end happening in between isn't missed.
        let mut updates = self.subscribe();
        let deadline = Instant::now() + timeout;
        loop {
            let status = self.status(job)?;
            if status.state.has_ended() {
                return Some(status);
            }

            match timeout_at(deadline, updates.recv()).await {
                Ok(Ok(_)) | Ok(Err(broadcast::error::RecvError::Lagged(_))) => continue,
                Ok(Err(broadcast::error::RecvError::Closed)) | Err(_) => return self.status(job),
            }
        }
    }

    /// Changes the state of a job with `change`, which returns the new state, or `None` to keep the current one.
    ///
    /// # Returns
    ///
    /// The new state of the job, or `None` if it's unknown or didn't change.
    fn transition(&self, job: u64, change: impl FnOnce(&JobState) -> Option<JobState>) -> Option<JobStatus> {
        let status = {
            let mut jobs = self.jobs.lock().unwrap();
            let record = jobs.get_mut(&job)?;
            record.status.state = change(&record.status.state)?;
            record.updated_at = SystemTime::now();
            record.status.clone()
        };

        self.notify(status.clone());
        Some(status)
    }

    /// Stores the state of a job, forgetting the oldest ended jobs over [`MAX_ENDED_JOBS`], and notifies it.
    fn set(&self, status: JobStatus) {
        {
            let mut jobs = self.jobs.lock().unwrap();
            let now = SystemTime::now();
            jobs.insert(status.job, JobRecord { status: status.clone(), created_at: now, updated_at: now });

            let ended = jobs.values().filter(|record| record.status.state.has_ended()).map(|record| record.status.job).collect::<Vec<_>>();
            for job in ended.iter().take(ended.len().saturating_sub(MAX_ENDED_JOBS)) {
                jobs.remove(job);
            }
        }

        self.notify(status);
    }

    /// Sends a job change to the listeners, if there are any.
    fn notify(&self, status: JobStatus) {
        let _ = self.updates.send(status);
    }
}

/// Returns if a method is one of the job control methods, which are answered by [`handle_request`].
pub fn is_job_control_method(method: &str) -> bool {
    matches!(method, GetJobStatus::METHOD | WaitForJob::METHOD | CancelJob::METHOD)
}

/// Answers a job control request, without waiting for the session's other requests.
///
/// # Arguments
///
/// * `registry` - Jobs of the session.
/// * `request` - A job control request. See [`is_job_control_method`].
/// * `sender` - Where to send the response.
pub fn handle_request(registry: Arc<JobRegistry>, request: RpcRequest, sender: UnboundedSender<RpcResponse>) {
    tokio::spawn(async move {
        let result = match request.method.as_str() {
            GetJobStatus::METHOD => parse_params::<GetJobStatus>(request.params)
                .and_then(|request| registry.status(request.job).ok_or(ApiError::JobNotFound(request.job)))
                .and_then(to_json),
            WaitForJob::METHOD => match parse_params::<WaitForJob>(request.params) {
                Ok(request) => {
                    let timeout = Duration::from_secs(request.timeout_secs.unwrap_or(DEFAULT_WAIT_SECS));
                    registry.wait(request.job, timeout).await.ok_or(ApiError::JobNotFound(request.job)).and_then(to_json)
                }
                Err(error) => Err(error),
            },
            CancelJob::METHOD => parse_params::<CancelJob>(request.params)
                .and_then(|request| registry.cancel(request.job))
                .and_then(|_| to_json(Done {})),
            method => Err(ApiError::MethodNotFound(method.to_owned())),
        };

        let _ = sender.send(RpcResponse::new(request.id, result));
    });
}

/// Serializes a response.
fn to_json(value: impl Serialize) -> Result<Value, ApiError> {
    serde_json::to_value(value).map_err(|error| ApiError::Internal(error.to_string()))
}

//-------------------------------------------------------------------------------//
//                                   Tests
//-------------------------------------------------------------------------------//

#[cfg(test)]
#[path = "jobs_test.rs"]
mod jobs_test;
