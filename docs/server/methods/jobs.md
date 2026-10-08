<!-- Generated from the API types by `rpfm_ipc/src/api/method_docs.rs`. Don't edit it by hand. -->

# Jobs

Methods to follow and cancel jobs.

Methods that can take a long time run as jobs (see `Request::IS_JOB`): they answer right away
with a `JobStarted`, and run in order with the rest of the session's requests. Their state is
sent to clients in `JOB_UPDATED_NOTIFICATION` notifications,
and can be checked or waited for with the methods here, which answer without waiting for other requests.

## `job.status`

Returns the state of a job.

**Params** (`GetJobStatus`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `job` | `integer` | yes | ID of the job. |

**Result** (`JobStatus`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `job` | `integer` | yes | ID of the job. |
| `method` | `string` | yes | Method the job runs. |

Plus one of:

- `{ "state": "queued" }`: Waiting for the requests before it to finish.
- `{ "state": "running", "progress"?: …, "stage"?: … }`: Running.
    - `progress` (`integer` or `null`, no): How far along the job is, from 0 to 100, if it reports it.
    - `stage` (`string` or `null`, no): Step the job is on, if it reports them.
- `{ "state": "finished", "result": … }`: Finished, with the response of its method.
    - `result` (`any`, yes): Response of the job's method.
- `{ "state": "failed", "error": … }`: Failed, with the error of its method.
    - `error` ([`RpcError`](#type-rpcerror), yes): Why the job failed.
- `{ "state": "cancelled" }`: Cancelled before it started.

## `job.wait`

Waits for a job to end, and returns its state.

**Params** (`WaitForJob`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `job` | `integer` | yes | ID of the job. |
| `timeout_secs` | `integer` or `null` | no | Maximum amount of seconds to wait. If the job hasn't ended by then, its current state is returned. Defaults to 60. |

**Result** (`JobStatus`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `job` | `integer` | yes | ID of the job. |
| `method` | `string` | yes | Method the job runs. |

Plus one of:

- `{ "state": "queued" }`: Waiting for the requests before it to finish.
- `{ "state": "running", "progress"?: …, "stage"?: … }`: Running.
    - `progress` (`integer` or `null`, no): How far along the job is, from 0 to 100, if it reports it.
    - `stage` (`string` or `null`, no): Step the job is on, if it reports them.
- `{ "state": "finished", "result": … }`: Finished, with the response of its method.
    - `result` (`any`, yes): Response of the job's method.
- `{ "state": "failed", "error": … }`: Failed, with the error of its method.
    - `error` ([`RpcError`](#type-rpcerror), yes): Why the job failed.
- `{ "state": "cancelled" }`: Cancelled before it started.

## `job.cancel`

Cancels a job that hasn't started yet. Running jobs can't be cancelled.

**Params** (`CancelJob`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `job` | `integer` | yes | ID of the job. |

**Result** (`Done`)

Empty object: `{}`.

## Types

### Type `RpcError`

A JSON-RPC error.

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `code` | `integer` | yes | Numeric code of the error. See `ApiError::code`. |
| `message` | `string` | yes | Human-readable description of the error. |
| `data` | `any` | no | The `ApiError` this was built from, serialized, so clients can match on it. |
