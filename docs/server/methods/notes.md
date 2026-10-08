<!-- Generated from the API types by `rpfm_ipc/src/api/method_docs.rs`. Don't edit it by hand. -->

# Notes

Methods about the notes of the open packs: comments attached to their files and folders.

## `notes.list`

Returns the notes of an open pack for a path.

**Params** (`ListNotes`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `pack` | `string` | yes | Key of the pack. |
| `path` | `string` | no, default `""` | Path of the file or folder whose notes to return, with the notes of the folders containing it. Empty for all the notes of the pack. |

**Result** (`NoteList`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `notes` | array of [`NoteEntry`](#type-noteentry) | yes | The notes. |

## `notes.add`

Attaches a note to a file or folder of an open pack.

**Params** (`AddNote`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `pack` | `string` | yes | Key of the pack. |
| `path` | `string` | no, default `""` | Path of the file or folder to attach the note to. Empty for the whole pack. |
| `message` | `string` | yes | Text of the note. |
| `url` | `string` or `null` | no | Link to attach to the note. |
| `id` | `integer` or `null` | no | ID of a note attached to the same path, to replace it instead of adding a new one. |

**Result** (`NoteEntry`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `id` | `integer` | yes | ID of the note, unique within its path. |
| `path` | `string` | yes | Path of the file or folder the note is attached to. Empty for the whole pack. |
| `message` | `string` | yes | Text of the note. |
| `url` | `string` or `null` | no | Link attached to the note, if any. |

## `notes.delete`

Deletes a note of an open pack.

**Params** (`DeleteNote`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `pack` | `string` | yes | Key of the pack. |
| `path` | `string` | yes | Path the note is attached to. |
| `id` | `integer` | yes | ID of the note. |

**Result** (`Done`)

Empty object: `{}`.

## Types

### Type `NoteEntry`

A note of a pack.

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `id` | `integer` | yes | ID of the note, unique within its path. |
| `path` | `string` | yes | Path of the file or folder the note is attached to. Empty for the whole pack. |
| `message` | `string` | yes | Text of the note. |
| `url` | `string` or `null` | no | Link attached to the note, if any. |
