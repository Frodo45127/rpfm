<!-- Generated from the API types by `rpfm_ipc/src/api/method_docs.rs`. Don't edit it by hand. -->

# Diagnostics

Methods to check the open packs for problems.

The server keeps the results of the last check, so clients get a summary when running it,
and read the results they need with `ListDiagnostics`.

## `diagnostics.run`

Checks the open packs for problems, keeping the results for `diagnostics.list`.

A running check stops when other requests arrive, and is queued again behind them. A new check replaces the
one that hasn't ended yet, which ends cancelled, and also checks the paths the replaced one would have checked.

Runs as a job: answers right away with a job ID, and sends its result when the job ends (see [Jobs](../ws-protocol.md#jobs)).

**Params** (`RunDiagnostics`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `paths` | array of `string` | no, default `[]` | If set, only files and folders of the open packs at these paths are checked again, keeping the results of the last check for the rest. If not set, or if there are no previous results, everything is checked. |
| `ignored_types` | array of `string` | no, default `[]` | Report types to skip, like `OutdatedTable` or `InvalidReference`. |
| `check_assembly_kit_only_references` | `boolean` | no, default `false` | If references to tables only in the Assembly Kit are checked too. |

**Result** (`DiagnosticsSummary`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `total` | `integer` | yes | Amount of results. |
| `by_level` | map of `integer` | yes | Amount of results of each level. |
| `by_type` | map of `integer` | yes | Amount of results of each report type. |

## `diagnostics.list`

Returns results of the last check, optionally filtered.

**Params** (`ListDiagnostics`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `levels` | array of [`DiagnosticLevel`](#type-diagnosticlevel) or `null` | no | If set, only results of these levels are returned. |
| `report_types` | array of `string` or `null` | no | If set, only results of these report types are returned. |
| `pack` | `string` or `null` | no | If set, only results of this pack are returned. |
| `path_prefix` | `string` | no, default `""` | Only results of files whose path starts with this are returned. |
| `offset` | `integer` | no, default `0` | Amount of matching results to skip. |
| `limit` | `integer` or `null` | no | Maximum amount of results to return. Defaults to 100. |

**Result** (`DiagnosticList`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `results` | array of [`DiagnosticResult`](#type-diagnosticresult) | yes | The results in the page. |
| `total` | `integer` | yes | Amount of results matching the request, in all pages. |

## `diagnostics.report`

Returns the results of the last diagnostics check, as the check returns them.

Meant for clients showing the results with all their details. Others should use `diagnostics.list`.

**Params** (`GetDiagnosticsReport`)

Empty object: `{}`.

**Result** (`Diagnostics`)

One of RPFM's own types, without a schema. See its docs in `rpfm_lib` or `rpfm_extensions`.

## `diagnostics.ignore`

Makes the next checks of a pack skip some results. Saved in the pack's settings.

**Params** (`IgnoreDiagnostics`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `pack` | `string` | yes | Key of the pack. |
| `path` | `string` | yes | Files whose path starts with this are affected. |
| `columns` | array of `string` | no, default `[]` | If set, only these columns of the files are affected. |
| `report_types` | array of `string` | no, default `[]` | If set, only these report types are skipped. Otherwise, everything is skipped. |

**Result** (`Done`)

Empty object: `{}`.

## Types

### Type `DiagnosticLevel`

Severity of a result.

One of:

- `"info"`: A suggestion, or information about the mod.
- `"warning"`: Something that may be a mistake, or cause problems in some situations.
- `"error"`: Something that will likely cause problems, like invalid references or malformed data.

### Type `DiagnosticResult`

A problem found in a check.

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `pack` | `string` | yes | Key of the pack with the problem. Empty for problems of the configuration. |
| `path` | `string` | yes | Path of the file with the problem. Empty for problems of a whole pack or of the configuration. |
| `checked` | `string` | yes | What was checked: `db`, `loc`, `text`, `pack`, `config`, `dependency`, `portrait_settings`, `group_formations` or `anim_fragment_battle`. |
| `report_type` | `string` | yes | Type of the problem, like `InvalidReference`. Used to ignore it. |
| `level` | [`DiagnosticLevel`](#type-diagnosticlevel) | yes | Severity of the problem. |
| `message` | `string` | yes | Description of the problem. |
| `cells` | array of array of `integer` | no | Table cells with the problem, as `[row, column]`. -1 means the whole row or column. |
| `columns` | array of `string` | no | Names of the columns of the cells with the problem. |
