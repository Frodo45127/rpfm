//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Methods to follow and cancel jobs.
//!
//! Methods that can take a long time run as jobs (see [`Request::IS_JOB`]): they answer right away
//! with a [`JobStarted`], and run in order with the rest of the session's requests. Their state is
//! sent to clients in [`JOB_UPDATED_NOTIFICATION`](super::JOB_UPDATED_NOTIFICATION) notifications,
//! and can be checked or waited for with the methods here, which answer without waiting for other requests.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{Done, Request, RpcError};

/// Default amount of seconds [`WaitForJob`] waits.
pub const DEFAULT_WAIT_SECS: u64 = 60;

/// Response of the methods that run as jobs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct JobStarted {

    /// ID of the job.
    pub job: u64,
}

/// State of a job.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct JobStatus {

    /// ID of the job.
    pub job: u64,

    /// Method the job runs.
    pub method: String,

    /// What the job is doing, or how it ended.
    #[serde(flatten)]
    pub state: JobState,
}

/// What a job is doing, or how it ended.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum JobState {

    /// Waiting for the requests before it to finish.
    Queued,

    /// Running.
    Running {

        /// Step the job is on, if it reports them.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        stage: Option<String>,

        /// How far along the job is, from 0 to 100, if it reports it.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        progress: Option<u8>,
    },

    /// Finished, with the response of its method.
    Finished {

        /// Response of the job's method.
        result: Value,
    },

    /// Failed, with the error of its method.
    Failed {

        /// Why the job failed.
        error: RpcError,
    },

    /// Cancelled before it started.
    Cancelled,
}

/// `job.status`: returns the state of a job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct GetJobStatus {

    /// ID of the job.
    pub job: u64,
}

/// `job.wait`: waits for a job to end, and returns its state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct WaitForJob {

    /// ID of the job.
    pub job: u64,

    /// Maximum amount of seconds to wait. If the job hasn't ended by then, its current state is returned.
    /// Defaults to [`DEFAULT_WAIT_SECS`].
    #[serde(default)]
    pub timeout_secs: Option<u64>,
}

/// `job.cancel`: cancels a job that hasn't started yet. Running jobs can't be cancelled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CancelJob {

    /// ID of the job.
    pub job: u64,
}

impl JobState {

    /// Returns if the job has ended, successfully or not.
    pub fn has_ended(&self) -> bool {
        matches!(self, Self::Finished { .. } | Self::Failed { .. } | Self::Cancelled)
    }
}

impl Request for GetJobStatus {
    const METHOD: &'static str = "job.status";
    type Response = JobStatus;
}

impl Request for WaitForJob {
    const METHOD: &'static str = "job.wait";
    type Response = JobStatus;
}

impl Request for CancelJob {
    const METHOD: &'static str = "job.cancel";
    type Response = Done;
}
