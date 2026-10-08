<!-- Generated from the API types by `rpfm_ipc/src/api/method_docs.rs`. Don't edit it by hand. -->

# Search

Methods to search text across the open packs, the dependencies and the schema, and replace it.

The server keeps the matches of the last search, so clients get a summary when running it,
read the matches they need with `ListSearchMatches`, and replace them by ID.

## `search.report`

Returns the last search with all its matches, as the search returns them.

Meant for clients showing the matches with all their details. Others should use `search.matches`.

**Params** (`GetSearchReport`)

Empty object: `{}`.

**Result** (`SearchReport`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `search` | `any` | yes | The search, with its matches. |
| `first_ids` | map of array of `integer` | yes | ID of the first match of each file, by type of file, in the order of the files in the search. The other matches of a file follow it, so its match number `n` has the ID `first + n`. |

## `search.run`

Searches text, keeping the matches for `search.matches`.

Runs as a job: answers right away with a job ID, and sends its result when the job ends (see [Jobs](../ws-protocol.md#jobs)).

**Params** (`RunSearch`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `pattern` | `string` | yes | Text to search, or a regular expression if `use_regex` is set. |
| `case_sensitive` | `boolean` | no, default `false` | If the search is case sensitive. |
| `use_regex` | `boolean` | no, default `false` | If the pattern is a regular expression. |
| `sources` | array of [`FileSource`](#type-filesource) | yes | Where to search: open packs, the game files, the parent packs, or the Assembly Kit tables. |
| `file_types` | array of `string` or `null` | no | Types of files to search, named like in `files.list`: `DB`, `Loc`, `Text`, `Atlas`, `PortraitSettings`, `RigidModel`, `UnitVariant`, `AnimFragmentBattle`, binary types like `Image`, `Audio` or `Unknown`, or `Schema` for the column names of the schema. Defaults to `DB`, `Loc` and `Text`. |

**Result** (`SearchSummary`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `total` | `integer` | yes | Amount of matches. |
| `files` | `integer` | yes | Amount of files with matches. |
| `by_type` | map of `integer` | yes | Amount of matches in each type of file. |

## `search.matches`

Returns matches of the last search, optionally filtered.

**Params** (`ListSearchMatches`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `file_types` | array of `string` or `null` | no | If set, only matches in these types of files are returned. |
| `path_prefix` | `string` | no, default `""` | Only matches in files whose path starts with this are returned. |
| `offset` | `integer` | no, default `0` | Amount of matching matches to skip. |
| `limit` | `integer` or `null` | no | Maximum amount of matches to return. Defaults to 100. |

**Result** (`SearchMatchList`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `matches` | array of [`SearchMatch`](#type-searchmatch) | yes | The matches in the page. |
| `total` | `integer` | yes | Amount of matches matching the request, in all pages. |

## `search.replace`

Replaces matches of the last search, and searches the edited files again.

Only matches in open packs can be replaced, and only in tables, text files, atlases, portrait
settings, rigid models, unit variants and anim fragment battles.

**Params** (`ReplaceSearchMatches`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `replace_text` | `string` | yes | Text to replace the matches with. |
| `matches` | array of `integer` or `null` | no | IDs of the matches to replace. If not set, all the matches are replaced. |

**Result** (`SearchReplaced`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `edited` | array of `string` | yes | Paths of the edited files. |
| `summary` | [`SearchSummary`](#type-searchsummary) | yes | Summary of the matches left after searching the edited files again. |

## Types

### Type `FileSource`

Where a file is.

One of:

- `{ "pack": … }`: An open pack, by key.
    - `pack` (`string`, yes)
- `"game_files"`: The vanilla files of the selected game.
- `"parent_files"`: The files of the parent packs of the open packs.
- `"assembly_kit"`: The tables of the selected game's Assembly Kit. Their paths are `db/<table_name>/ak_data`.

### Type `SearchMatch`

A match of a search.

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `id` | `integer` | yes | ID of the match, used to replace it. IDs change after every search and replace. |
| `source` | [`FileSource`](#type-filesource) or `null` | no | Where the file with the match is. Not set for matches in the schema. |
| `path` | `string` | yes | Path of the file with the match. Empty for matches in the schema. |
| `file_type` | `string` | yes | Type of the file with the match, like `DB` or `Text`. |
| `details` | `any` | yes | Where the match is in its file, and its text. The fields depend on the type of the file: tables have `column_name`, `row_number`, `start`, `end` and `text`, text files have `row`, `start`, `end` and `text`, and so on. |

### Type `SearchSummary`

Summary of the matches of a search.

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `total` | `integer` | yes | Amount of matches. |
| `files` | `integer` | yes | Amount of files with matches. |
| `by_type` | map of `integer` | yes | Amount of matches in each type of file. |
