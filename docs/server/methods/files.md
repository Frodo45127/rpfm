<!-- Generated from the API types by `rpfm_ipc/src/api/method_docs.rs`. Don't edit it by hand. -->

# Files

Methods about the files of the open packs and the dependencies.

## `files.list`

Lists the files of a source, sorted by path.

**Params** (`ListFiles`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `source` | [`FileSource`](#type-filesource) | yes | Source to list. |
| `path_prefix` | `string` | no, default `""` | Only files whose path starts with this are listed. |
| `recursive` | `boolean` | no, default `true` | If files in subfolders are listed. If `false`, the subfolders are listed instead. |
| `file_types` | array of `string` or `null` | no | If set, only files of these types are listed, like `DB`, `Loc`, `Text`, `Image` or `RigidModel`. |
| `offset` | `integer` | no, default `0` | Amount of files to skip. |
| `limit` | `integer` or `null` | no | Maximum amount of files to return. Defaults to 500. |

**Result** (`FileList`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `files` | array of [`FileEntry`](#type-fileentry) | yes | The files in the page. |
| `folders` | array of `string` | yes | Subfolders directly under the prefix, with their full path. Only filled in non-recursive listings. |
| `total` | `integer` | yes | Amount of files matching the request, in all pages. |

## `files.create`

Creates a new empty file in an open pack.

**Params** (`CreateFile`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `pack` | `string` | yes | Key of the pack. |
| `path` | `string` | yes | Path of the new file in the pack, like `db/units_tables/my_mod` or `text/db/my_mod.loc`. |
| `kind` | [`NewFileKind`](#type-newfilekind) | yes | Type of the new file. |

**Result** (`FileEntry`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `path` | `string` | yes | Path of the file in its source. |
| `file_type` | `string` | yes | Type of the file, like `DB`, `Loc`, `Text` or `Image`. |

## `files.add_from_disk`

Adds files and folders from disk to an open pack.

**Params** (`AddFilesFromDisk`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `pack` | `string` | yes | Key of the pack. |
| `paths` | array of `string` | yes | Paths on disk of the files and folders to add. |
| `folder` | `string` | no, default `""` | Folder of the pack to add them to. Empty for the root of the pack. |
| `pack_paths` | array of `string` or `null` | no | Path in the pack of each entry of `paths`, in the same order, instead of putting them all in `folder`. For files, it's their new path. For folders, it's the folder their contents go to. |
| `include_base_folder` | `boolean` or `null` | no | If added folders keep their own name in the pack, instead of adding only their contents. Defaults to the server's setting. |
| `ignore` | array of `string` | no, default `[]` | Files and folders under any of these paths are skipped. |

**Result** (`FilesAdded`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `added` | array of `string` | yes | Paths in the pack of the added files. |
| `not_added` | array of `string` | no | Paths of the files that couldn't be added. |
| `error` | `string` or `null` | no | Last error found while adding the files, if any. |

## `files.copy`

Copies files and folders from any source into an open pack, keeping their paths.

**Params** (`CopyFiles`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `source` | [`FileSource`](#type-filesource) | yes | Where to copy the files from. |
| `paths` | array of `string` | yes | Paths of the files and folders to copy. |
| `pack` | `string` | yes | Key of the pack to copy them to. |

**Result** (`FilesAdded`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `added` | array of `string` | yes | Paths in the pack of the added files. |
| `not_added` | array of `string` | no | Paths of the files that couldn't be added. |
| `error` | `string` or `null` | no | Last error found while adding the files, if any. |

## `files.delete`

Deletes files and folders from an open pack.

**Params** (`DeleteFiles`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `pack` | `string` | yes | Key of the pack. |
| `paths` | array of `string` | yes | Paths of the files and folders to delete. |

**Result** (`FilesDeleted`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `deleted` | array of `string` | yes | Paths of the deleted files. |

## `files.rename`

Renames or moves files and folders of an open pack.

**Params** (`RenameFiles`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `pack` | `string` | yes | Key of the pack. |
| `renames` | array of [`FileRename`](#type-filerename) | yes | Renames to apply. |

**Result** (`FilesRenamed`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `renamed` | array of [`FileRename`](#type-filerename) | yes | Old and new path of each renamed file. |

## `files.duplicate`

Copies files of an open pack in the same pack, adding a number to their names.

**Params** (`DuplicateFiles`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `pack` | `string` | yes | Key of the pack. |
| `paths` | array of `string` | yes | Paths of the files and folders to duplicate. |

**Result** (`FilesAdded`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `added` | array of `string` | yes | Paths in the pack of the added files. |
| `not_added` | array of `string` | no | Paths of the files that couldn't be added. |
| `error` | `string` or `null` | no | Last error found while adding the files, if any. |

## `files.extract`

Extracts files and folders of an open pack, the game files or the parent packs to disk.

**Params** (`ExtractFiles`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `source` | [`FileSource`](#type-filesource) | yes | Where the files are. The Assembly Kit tables can't be extracted. |
| `paths` | array of `string` | yes | Paths of the files and folders to extract. |
| `destination` | `string` | yes | Folder on disk to extract them to. Their paths in the source are kept under it. |
| `as_tsv` | `boolean` | no, default `false` | If tables are extracted as TSV files instead of binary ones. |
| `tsv_keys_first` | `boolean` or `null` | no | If TSV files use the old column order, with keys first. Defaults to the server's setting. |

**Result** (`FilesExtracted`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `extracted` | array of `string` | yes | Paths on disk of the extracted files. |

## `file.read`

Returns the contents of a file of any source.

**Params** (`ReadFile`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `file` | [`FileRef`](#type-fileref) | yes | The file. |
| `format` | [`ReadFormat`](#type-readformat) | no, default `"decoded"` | How to return the contents. Defaults to `decoded`. |

**Result** (`FileContents`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `file_type` | `string` | yes | Type of the file, like `Text`, `Image` or `RigidModel`. |
| `contents` | [`FileData`](#type-filedata) | yes | The contents. |

## `file.write`

Replaces the contents of a file of an open pack.

**Params** (`WriteFile`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `pack` | `string` | yes | Key of the pack. |
| `path` | `string` | yes | Path of the file. With raw contents, the file is created if it doesn't exist. |
| `contents` | [`FileData`](#type-filedata) | yes | The new contents. Text contents only work on files that are text files. |

**Result** (`Done`)

Empty object: `{}`.

## `animpack.list`

Lists the files inside an AnimPack of any source.

**Params** (`ListAnimPack`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `file` | [`FileRef`](#type-fileref) | yes | The AnimPack. |

**Result** (`FileList`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `files` | array of [`FileEntry`](#type-fileentry) | yes | The files in the page. |
| `folders` | array of `string` | yes | Subfolders directly under the prefix, with their full path. Only filled in non-recursive listings. |
| `total` | `integer` | yes | Amount of files matching the request, in all pages. |

## `animpack.add`

Copies files of an open pack into an AnimPack of an open pack.

**Params** (`AddToAnimPack`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `pack` | `string` | yes | Key of the pack with the AnimPack. |
| `animpack` | `string` | yes | Path of the AnimPack. |
| `source_pack` | `string` | yes | Key of the pack to copy the files from. |
| `paths` | array of `string` | yes | Paths of the files and folders to copy. |

**Result** (`FilesAdded`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `added` | array of `string` | yes | Paths in the pack of the added files. |
| `not_added` | array of `string` | no | Paths of the files that couldn't be added. |
| `error` | `string` or `null` | no | Last error found while adding the files, if any. |

## `animpack.extract`

Copies files of an AnimPack of any source into an open pack.

**Params** (`ExtractFromAnimPack`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `file` | [`FileRef`](#type-fileref) | yes | The AnimPack. |
| `paths` | array of `string` | yes | Paths inside the AnimPack of the files and folders to copy. |
| `pack` | `string` | yes | Key of the pack to copy them to. |

**Result** (`FilesAdded`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `added` | array of `string` | yes | Paths in the pack of the added files. |
| `not_added` | array of `string` | no | Paths of the files that couldn't be added. |
| `error` | `string` or `null` | no | Last error found while adding the files, if any. |

## `animpack.delete`

Deletes files from an AnimPack of an open pack.

**Params** (`DeleteFromAnimPack`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `pack` | `string` | yes | Key of the pack with the AnimPack. |
| `animpack` | `string` | yes | Path of the AnimPack. |
| `paths` | array of `string` | yes | Paths inside the AnimPack of the files and folders to delete. |

**Result** (`Done`)

Empty object: `{}`.

## `file.view_data`

Returns a file decoded as a client opening it in a view needs it.

Meant for clients with views for each type of file. Others should use `file.read`.

**Params** (`GetViewData`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `file` | [`FileRef`](#type-fileref) | yes | The file. |

**Result** (`ViewData`)

One of:

- `{ "Decoded": … }`: The decoded file, and its info.
    - `Decoded` ([`any`, `any`], yes)
- `{ "AnimPack": … }`: The info of the files of an AnimPack, and of the AnimPack itself.
    - `AnimPack` ([`any`, `any`], yes)
- `{ "Video": … }`: The info of a video, without its frames, and of the file.
    - `Video` ([`any`, `any`], yes)
- `{ "Notes": … }`: The notes of a pack, as markdown.
    - `Notes` (`any`, yes)
- `"Unsupported"`: The file is of a type that is not decoded.

## `files.info`

Returns the info of files of an open pack, like their original pack and last modification time.

Meant for clients showing the files of a pack as a tree. Others should use `files.list`.

**Params** (`GetFilesInfo`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `pack` | `string` | yes | Key of the pack. |
| `paths` | array of `string` or `null` | no | Paths of the files. If not set, the info of every file of the pack is returned. Paths without a file are skipped. |

**Result** (`FilesInfo`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `files` | array of `any` | yes | The info of each file. |

## `files.paste`

Copies or moves files and folders of open packs into a folder of an open pack.

Each selected file or folder is pasted with its own name under the destination folder, not with its full path.
Meant for clients with a clipboard. Others should use `files.copy` and `files.rename`.

**Params** (`PasteFiles`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `sources` | map of array of `string` | yes | Paths of the files and folders to paste, by key of their pack. |
| `cut` | `boolean` | yes | If the files are removed from their packs, instead of copied. |
| `pack` | `string` | yes | Key of the pack to paste them into. |
| `folder` | `string` | yes | Folder of the pack to paste them into. Empty for the root of the pack. |

**Result** (`FilesPasted`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `added` | array of `string` | yes | Paths of the files added to the target pack. |
| `deleted` | map of array of `string` | yes | Paths of the files removed from each source pack, by pack key, if they were cut. |

## `files.from_all_sources`

Returns files and folders found in the open packs, the parent packs and the game files.

Meant for clients editing files of several sources at once, like the UI's tools. Others should use `file.read`.

**Params** (`GetFilesFromAllSources`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `paths` | array of `any` | yes | Paths of the files and folders. |
| `lowercase_paths` | `boolean` | no, default `false` | If the paths of the returned files are lowercased. |

**Result** (`FilesFromAllSources`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `files` | `any` | yes | The files found, by path, for each source. |

## `files.save_files`

Adds files to an open pack, replacing the ones with the same path, optionally optimizing the pack after it.

Meant for clients editing files outside of the server, like the UI's tools. Others should use `file.write`.

**Params** (`SaveFiles`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `pack` | `string` | yes | Key of the pack. |
| `files` | array of `any` | yes | The files, named with their path in the pack. |
| `optimize` | `boolean` | no, default `false` | If the pack is optimized with the optimizer options of the settings after adding the files. |

**Result** (`FilesChanged`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `added` | array of `string` | yes | Paths of the added or edited files. |
| `deleted` | array of `string` | yes | Paths of the deleted files. |

## `file.open_external`

Extracts a file of an open pack to a temporary folder, and opens it in the system's default program for its type.

**Params** (`OpenInExternalProgram`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `pack` | `string` | yes | Key of the pack. |
| `path` | `string` | yes | Path of the file. |

**Result** (`ExternalFile`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `path` | `string` | yes | Path of the extracted file on disk. |

## `file.save_external`

Replaces a file of an open pack with the file on disk an external program edited.

**Params** (`SaveExternalFile`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `pack` | `string` | yes | Key of the pack. |
| `path` | `string` | yes | Path of the file in the pack. |
| `external_path` | `string` | yes | Path of the edited file on disk. |

**Result** (`Done`)

Empty object: `{}`.

## Types

### Type `FileData`

Contents of a file.

One of:

- `{ "kind": "text", "text": … }`: The text of a text file.
    - `text` (`string`, yes): The text.
- `{ "kind": "decoded", "data": … }`: The decoded file, as JSON, in the format `file.read` returns it.
    - `data` (`any`, yes): The decoded file.
- `{ "kind": "raw", "base64": … }`: The bytes of the file.
    - `base64` (`string`, yes): The bytes, encoded in base64.

### Type `FileEntry`

A file in a listing.

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `path` | `string` | yes | Path of the file in its source. |
| `file_type` | `string` | yes | Type of the file, like `DB`, `Loc`, `Text` or `Image`. |

### Type `FileRef`

A file in one of the sources.

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `source` | [`FileSource`](#type-filesource) | yes | Where the file is. |
| `path` | `string` | yes | Path of the file in its source. |

### Type `FileRename`

A rename of a file or folder.

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `from` | `string` | yes | Current path. |
| `to` | `string` | yes | New path. |

### Type `FileSource`

Where a file is.

One of:

- `{ "pack": … }`: An open pack, by key.
    - `pack` (`string`, yes)
- `"game_files"`: The vanilla files of the selected game.
- `"parent_files"`: The files of the parent packs of the open packs.
- `"assembly_kit"`: The tables of the selected game's Assembly Kit. Their paths are `db/<table_name>/ak_data`.

### Type `NewFileKind`

Type of a new file.

One of:

- `{ "type": "db", "table_name": …, "version"?: … }`: An empty DB table.
    - `table_name` (`string`, yes): Name of the table, like `units_tables`.
    - `version` (`integer` or `null`, no): Version of the table. If not set, the version the table has in the game files is used, or the newest one if the game files are not loaded or don't have the table.
- `{ "type": "loc" }`: An empty Loc table.
- `{ "type": "text", "format"?: … }`: An empty text file.
    - `format` (`string` or `null`, no): Format of the text, like `Plain`, `Lua`, `Xml` or `Json`. Defaults to `Plain`.
- `{ "type": "anim_pack" }`: An empty AnimPack.
- `{ "type": "portrait_settings", "version": …, "copy_entries"?: … }`: A portrait settings file, optionally with entries copied from the vanilla ones.
    - `version` (`integer`, yes): Version of the file format.
    - `copy_entries` (array of [`EntryCopy`](#type-entrycopy), no, default `[]`): Vanilla entries to copy into the new file, with the ID each copy gets.
- `{ "type": "vmd" }`: An empty VMD (variant mesh definition) file.
- `{ "type": "ws_model" }`: An empty WSModel file.

### Type `EntryCopy`

An entry copied from a vanilla file, with a new ID.

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `from` | `string` | yes | ID of the vanilla entry. |
| `to` | `string` | yes | ID of the copy. |

### Type `ReadFormat`

How to return the contents of a file.

One of:

- `"decoded"`: The decoded file, as JSON. For DB and Loc tables, `table.rows` returns only the rows you need, and for images and other binary files `raw` is smaller.
- `"text"`: The text, for text files like scripts, XML or JSON files.
- `"raw"`: The bytes of the file, encoded in base64.
