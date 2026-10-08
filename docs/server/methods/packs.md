<!-- Generated from the API types by `rpfm_ipc/src/api/method_docs.rs`. Don't edit it by hand. -->

# Packs

Methods about the open packs.

## `pack.info`

Returns the details of an open pack.

**Params** (`GetPackInfo`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `pack` | `string` | yes | Key of the pack. |

**Result** (`PackDetails`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `version` | `string` | yes | Version of the pack format, like `PFH5`. |
| `compression` | `string` | yes | Compression format of the files of the pack, like `None` or `Lz4`. |
| `index_encrypted` | `boolean` | yes | If the index of the pack is encrypted. |
| `data_encrypted` | `boolean` | yes | If the data of the pack is encrypted. |
| `index_includes_timestamp` | `boolean` | yes | If the index of the pack includes the timestamp of each file. |
| `extended_header` | `boolean` | yes | If the pack has the extended header of some Arena packs. |
| `timestamp` | `integer` | yes | Unix time of the last time the pack was saved. |
| `dependencies` | array of [`PackDependency`](#type-packdependency) | yes | Packs this pack depends on, loaded as parent files. |
| `operational_mode` | `any` | yes | If the pack is in normal mode (`"Normal"`) or MyMod mode (`{"MyMod": [game_folder, mod_name]}`). |
| `file_count` | `integer` | yes | Amount of files in the pack. |
| `key` | `string` | yes | Key identifying the pack in every method that works on it: a UUID, like `0b4c5a62-7d3e-4f1a-9c2b-8e6d5f4a3b21`. It never changes while the pack is open. |
| `name` | `string` | yes | File name of the pack, like `my_mod.pack`. |
| `pack_type` | `string` | yes | Type of the pack: `Boot`, `Release`, `Patch`, `Mod` or `Movie`. |
| `path` | `string` or `null` | yes | Path of the pack on disk, if it's saved there. |

## `pack.new`

Creates a new empty pack. It has no path on disk until it's saved with a path.

**Params** (`NewPack`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `name` | `string` or `null` | no | File name of the pack, like `my_mod.pack`. Defaults to `new_pack.pack`, numbered if there's already an open pack with that name. |

**Result** (`PackSummary`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `key` | `string` | yes | Key identifying the pack in every method that works on it: a UUID, like `0b4c5a62-7d3e-4f1a-9c2b-8e6d5f4a3b21`. It never changes while the pack is open. |
| `name` | `string` | yes | File name of the pack, like `my_mod.pack`. |
| `path` | `string` or `null` | yes | Path of the pack on disk, if it's saved there. |
| `pack_type` | `string` | yes | Type of the pack: `Boot`, `Release`, `Patch`, `Mod` or `Movie`. |
| `file_count` | `integer` | yes | Amount of files in the pack. |

## `pack.open`

Opens one or more packs from disk, merged into a single one.

**Params** (`OpenPack`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `paths` | array of `string` | yes | Paths of the packs on disk. If there are more than one, the merged pack has no path, and is named after the first one. |
| `lazy_loading` | `boolean` or `null` | no | If file data is read from disk only when needed. Defaults to the server's setting. |

**Result** (`PackSummary`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `key` | `string` | yes | Key identifying the pack in every method that works on it: a UUID, like `0b4c5a62-7d3e-4f1a-9c2b-8e6d5f4a3b21`. It never changes while the pack is open. |
| `name` | `string` | yes | File name of the pack, like `my_mod.pack`. |
| `path` | `string` or `null` | yes | Path of the pack on disk, if it's saved there. |
| `pack_type` | `string` | yes | Type of the pack: `Boot`, `Release`, `Patch`, `Mod` or `Movie`. |
| `file_count` | `integer` | yes | Amount of files in the pack. |

## `pack.open_vanilla`

Opens all the vanilla packs of the selected game, merged into a single one.

**Params** (`OpenVanillaPacks`)

Empty object: `{}`.

**Result** (`PackSummary`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `key` | `string` | yes | Key identifying the pack in every method that works on it: a UUID, like `0b4c5a62-7d3e-4f1a-9c2b-8e6d5f4a3b21`. It never changes while the pack is open. |
| `name` | `string` | yes | File name of the pack, like `my_mod.pack`. |
| `path` | `string` or `null` | yes | Path of the pack on disk, if it's saved there. |
| `pack_type` | `string` | yes | Type of the pack: `Boot`, `Release`, `Patch`, `Mod` or `Movie`. |
| `file_count` | `integer` | yes | Amount of files in the pack. |

## `pack.close`

Closes an open pack, discarding unsaved changes.

**Params** (`ClosePack`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `pack` | `string` | yes | Key of the pack. |

**Result** (`Done`)

Empty object: `{}`.

## `pack.close_all`

Closes all open packs, discarding unsaved changes.

**Params** (`CloseAllPacks`)

Empty object: `{}`.

**Result** (`Done`)

Empty object: `{}`.

## `pack.save`

Saves an open pack to disk.

**Params** (`SavePack`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `pack` | `string` | yes | Key of the pack. |
| `path` | `string` or `null` | no | Absolute path to save the pack to, creating its missing folders. If not set, the pack is saved to its current path. |
| `clean` | `boolean` | no, default `false` | If files that failed to decode are removed before saving. Use it if saving normally fails. |
| `disable_uuid_regeneration` | `boolean` or `null` | no | If tables keep their GUID instead of getting a new one. Defaults to the server's setting. |
| `allow_editing_ca_packs` | `boolean` or `null` | no | If packs of the vanilla types (not `Mod` or `Movie`) can be saved. Defaults to the server's setting. |

**Result** (`PackSummary`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `key` | `string` | yes | Key identifying the pack in every method that works on it: a UUID, like `0b4c5a62-7d3e-4f1a-9c2b-8e6d5f4a3b21`. It never changes while the pack is open. |
| `name` | `string` | yes | File name of the pack, like `my_mod.pack`. |
| `path` | `string` or `null` | yes | Path of the pack on disk, if it's saved there. |
| `pack_type` | `string` | yes | Type of the pack: `Boot`, `Release`, `Patch`, `Mod` or `Movie`. |
| `file_count` | `integer` | yes | Amount of files in the pack. |

## `pack.update`

Changes properties of an open pack. Only the fields set are changed.

**Params** (`UpdatePack`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `pack` | `string` | yes | Key of the pack. |
| `pack_type` | `string` or `null` | no | New type of the pack: `Boot`, `Release`, `Patch`, `Mod` or `Movie`. |
| `compression` | `string` or `null` | no | New compression format of the pack, like `None` or `Lz4`. If the game doesn't support it, a supported one is used. |
| `index_encrypted` | `boolean` or `null` | no | If the index of the pack is encrypted. Only PFH4 and newer packs support encryption. |
| `data_encrypted` | `boolean` or `null` | no | If the data of the pack is encrypted. Only PFH4 and newer packs support encryption. |
| `index_includes_timestamp` | `boolean` or `null` | no | If the index of the pack includes the timestamp of each file. |
| `dependencies` | array of [`PackDependency`](#type-packdependency) or `null` | no | New list of packs this pack depends on. |
| `operational_mode` | `any` | no | New mode of the pack: `"Normal"`, or `{"MyMod": [game_folder, mod_name]}`. |

**Result** (`PackDetails`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `version` | `string` | yes | Version of the pack format, like `PFH5`. |
| `compression` | `string` | yes | Compression format of the files of the pack, like `None` or `Lz4`. |
| `index_encrypted` | `boolean` | yes | If the index of the pack is encrypted. |
| `data_encrypted` | `boolean` | yes | If the data of the pack is encrypted. |
| `index_includes_timestamp` | `boolean` | yes | If the index of the pack includes the timestamp of each file. |
| `extended_header` | `boolean` | yes | If the pack has the extended header of some Arena packs. |
| `timestamp` | `integer` | yes | Unix time of the last time the pack was saved. |
| `dependencies` | array of [`PackDependency`](#type-packdependency) | yes | Packs this pack depends on, loaded as parent files. |
| `operational_mode` | `any` | yes | If the pack is in normal mode (`"Normal"`) or MyMod mode (`{"MyMod": [game_folder, mod_name]}`). |
| `file_count` | `integer` | yes | Amount of files in the pack. |
| `key` | `string` | yes | Key identifying the pack in every method that works on it: a UUID, like `0b4c5a62-7d3e-4f1a-9c2b-8e6d5f4a3b21`. It never changes while the pack is open. |
| `name` | `string` | yes | File name of the pack, like `my_mod.pack`. |
| `pack_type` | `string` | yes | Type of the pack: `Boot`, `Release`, `Patch`, `Mod` or `Movie`. |
| `path` | `string` or `null` | yes | Path of the pack on disk, if it's saved there. |

## `pack.settings`

Returns the settings of an open pack.

**Params** (`GetPackSettings`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `pack` | `string` | yes | Key of the pack. |

**Result** (`PackSettingsValues`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `text` | map of `string` | no, default `{}` | Multi-line text settings. |
| `string` | map of `string` | no, default `{}` | Single-line text settings. |
| `bool` | map of `boolean` | no, default `{}` | Boolean settings. |
| `number` | map of `integer` | no, default `{}` | Number settings. |

## `pack.update_settings`

Changes settings of an open pack. Only the keys set are changed.

**Params** (`UpdatePackSettings`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `pack` | `string` | yes | Key of the pack. |
| `settings` | [`PackSettingsValues`](#type-packsettingsvalues) | yes | Settings to set, in the format `pack.settings` returns them. |

**Result** (`PackSettingsValues`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `text` | map of `string` | no, default `{}` | Multi-line text settings. |
| `string` | map of `string` | no, default `{}` | Single-line text settings. |
| `bool` | map of `boolean` | no, default `{}` | Boolean settings. |
| `number` | map of `integer` | no, default `{}` | Number settings. |

## Types

### Type `PackDependency`

A pack another pack depends on.

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `enabled` | `boolean` | yes | If the dependency is loaded. |
| `name` | `string` | yes | File name of the pack. |

### Type `PackSettingsValues`

Settings of a pack, by type of value and key, like `diagnostics_files_to_ignore` or `disable_autosaves`.

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `text` | map of `string` | no, default `{}` | Multi-line text settings. |
| `string` | map of `string` | no, default `{}` | Single-line text settings. |
| `bool` | map of `boolean` | no, default `{}` | Boolean settings. |
| `number` | map of `integer` | no, default `{}` | Number settings. |
