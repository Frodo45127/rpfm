<!-- Generated from the API types by `rpfm_ipc/src/api/method_docs.rs`. Don't edit it by hand. -->

# GitHub

Methods to sign in to GitHub, needed to submit translations to the Translation Hub.

The sign-in uses GitHub's device flow: the user enters a code on GitHub while the client polls for the result.
The token is kept by the server in the OS keyring, and never sent to clients.

## `github.sign_in_start`

Starts a GitHub sign-in, returning the code the user has to enter on GitHub.

**Params** (`StartGitHubSignIn`)

Empty object: `{}`.

**Result** (`DeviceCode`)

One of RPFM's own types, without a schema. See its docs in `rpfm_lib` or `rpfm_extensions`.

## `github.sign_in_poll`

Checks if the user approved a sign-in. Once approved, the server keeps the sign-in.

**Params** (`PollGitHubSignIn`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `device_code` | `string` | yes | Device code of the sign-in, from `github.sign_in_start`. |

**Result** (`GitHubSignInState`)

One of:

- `"Pending"`: The user hasn't approved the sign-in yet.
- `{ "SlowDown": … }`: Polling too fast. Contains the new minimum seconds between polls.
    - `SlowDown` (`integer`, yes)
- `{ "SignedIn": … }`: Signed in. Contains the account's login.
    - `SignedIn` (`string`, yes)
- `"Expired"`: The code expired before the user approved it.
- `"Denied"`: The user rejected the sign-in.

## `github.account`

Returns the GitHub account the user is signed in as.

**Params** (`GetGitHubAccount`)

Empty object: `{}`.

**Result** (`GitHubAccount`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `login` | `string` or `null` | yes | Login of the account, or `None` if the user isn't signed in. |

## `github.sign_out`

Signs out of GitHub, deleting the stored sign-in.

**Params** (`SignOutOfGitHub`)

Empty object: `{}`.

**Result** (`Done`)

Empty object: `{}`.
