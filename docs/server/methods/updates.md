<!-- Generated from the API types by `rpfm_ipc/src/api/method_docs.rs`. Don't edit it by hand. -->

# Updates

Methods to check for and download updates of RPFM and of the data it downloads.

These methods don't touch the session's state, so they run without waiting for its other requests.
Schemas are updated with `UpdateSchemas`, as they need reloading.

## `updates.check`

Checks if there is an update.

**Params** (`CheckUpdate`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `component` | [`UpdateComponent`](#type-updatecomponent) | yes | What to check. |

**Result** (`UpdateStatus`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `available` | `boolean` | yes | If there is an update. |
| `state` | [`UpdateState`](#type-updatestate) | yes | Details of the state. |
| `version` | `string` or `null` | no | Version of the update, for program updates. |

## `updates.apply`

Downloads an update. Schemas can't be updated with this, as they need reloading: use `schema.update`.

**Params** (`ApplyUpdate`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `component` | [`UpdateComponent`](#type-updatecomponent) | yes | What to update. |

**Result** (`Done`)

Empty object: `{}`.

## Types

### Type `UpdateComponent`

Something that can be updated.

One of:

- `"program"`: RPFM itself. Updating it replaces its files with the latest release, and needs a restart.
- `"schemas"`: The schemas with the table definitions of every game.
- `"lua_autogen"`: The Lua type definitions used for editor support in MyMods.
- `"old_assembly_kit"`: The Assembly Kit data of Empire and Napoleon, which don't have one.
- `"translations"`: The community translations of the Translation Hub.

### Type `UpdateState`

Details of the state of the updates of something.

One of:

- `"new_stable_update"`: There is a new stable release of the program.
- `"new_beta_update"`: There is a new beta release of the program.
- `"new_update_hotfix"`: There is a new hotfix release of the program.
- `"unknown_version"`: The program's version couldn't be compared with the latest release.
- `"new_update"`: There is a new version of the files.
- `"no_local_files"`: The files were never downloaded.
- `"diverged"`: The local files diverged from the remote ones, and need to be downloaded again.
- `"no_update"`: Everything is up to date.
