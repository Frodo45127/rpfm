<!-- Generated from the API types by `rpfm_ipc/src/api/method_docs.rs`. Don't edit it by hand. -->

# Tools

Methods of the tools: optimizer, maps, startpos, animations, glTF export, MyMods and Lua tests.

## `tools.optimizer_options`

Returns the optimizer options, as the server's settings have them.

**Params** (`GetOptimizerOptions`)

Empty object: `{}`.

**Result** (`OptimizerOptionValues`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `options` | map of `boolean` | yes | If each optimization is enabled, by name. |

## `tools.optimize`

Removes data of an open pack that's identical to the vanilla one, or unneeded.

Runs as a job: answers right away with a job ID, and sends its result when the job ends (see [Jobs](../ws-protocol.md#jobs)).

**Params** (`OptimizePack`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `pack` | `string` | yes | Key of the pack. |
| `options` | map of `boolean` | no, default `{}` | Optimizations to enable or disable, by name. The rest keep the server's settings. See `tools.optimizer_options` for the names. |

**Result** (`FilesChanged`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `added` | array of `string` | yes | Paths of the added or edited files. |
| `deleted` | array of `string` | yes | Paths of the deleted files. |

## `tools.patch_siege_ai`

Patches the siege maps of an open pack, so the AI can use them. Warhammer games only.

**Params** (`PatchSiegeAi`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `pack` | `string` | yes | Key of the pack. |

**Result** (`SiegeAiPatched`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `message` | `string` | yes | Description of what was patched. |
| `deleted` | array of `string` | yes | Paths of the files deleted by the patch. |

## `tools.pack_map`

Adds the tiles and tile maps of a map exported by Terry to an open pack.

**Params** (`PackMap`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `pack` | `string` | yes | Key of the pack. |
| `tile_maps` | array of `string` | yes | Paths on disk of the tile maps. |
| `tiles` | array of [`MapTile`](#type-maptile) | no, default `[]` | Tiles to add. |

**Result** (`FilesChanged`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `added` | array of `string` | yes | Paths of the added or edited files. |
| `deleted` | array of `string` | yes | Paths of the deleted files. |

## `tools.generate_missing_locs`

Adds empty loc entries for the localised columns of the tables of the open packs that have none.

**Params** (`GenerateMissingLocs`)

Empty object: `{}`.

**Result** (`FilesEdited`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `edited` | array of `string` | yes | Paths of the edited files. |

## `tools.update_anim_ids`

Offsets the animation ids of an open pack, like after a game update moves them.

**Params** (`UpdateAnimIds`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `pack` | `string` | yes | Key of the pack. |
| `starting_id` | `integer` | yes | Only ids from this one are changed. |
| `id_offset` | `integer` | yes | Amount to add to each id. |

**Result** (`FilesEdited`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `edited` | array of `string` | yes | Paths of the edited files. |

## `tools.anims_by_skeleton`

Returns the paths of the animations using a skeleton, in the open packs and the dependencies.

**Params** (`AnimsBySkeleton`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `skeleton` | `string` | yes | Name of the skeleton. |

**Result** (`FilePaths`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `paths` | array of `string` | yes | The paths, sorted. |

## `tools.export_gltf`

Exports a RigidModel to a glTF file, with its textures.

**Params** (`ExportGltf`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `file` | [`FileRef`](#type-fileref) | yes | The RigidModel. |
| `destination` | `string` | yes | Path of the glTF file to write. |

**Result** (`Done`)

Empty object: `{}`.

## `tools.set_video_format`

Changes the format of a ca_vp8 video of an open pack.

**Params** (`SetVideoFormat`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `pack` | `string` | yes | Key of the pack. |
| `path` | `string` | yes | Path of the video. |
| `format` | `string` | yes | New format: `CaVp8` or `Ivf`. |

**Result** (`Done`)

Empty object: `{}`.

## `tools.live_export`

Exports the scripts and UI files of an open pack to the game's data folder, to test them without saving the pack.

**Params** (`LiveExport`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `pack` | `string` | yes | Key of the pack. |

**Result** (`Done`)

Empty object: `{}`.

## `tools.init_mymod`

Creates the folder of a new MyMod, with editor configs for Lua scripting.

**Params** (`InitMyMod`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `game_folder` | `string` | yes | Folder name of the game of the MyMod, like `warhammer_3`. |
| `name` | `string` | yes | Name of the MyMod. |
| `sublime_support` | `boolean` | no, default `false` | If a Sublime Text project is created. |
| `vscode_support` | `boolean` | no, default `false` | If a VSCode config is created. |
| `gitignore` | `string` or `null` | no | If set, a git repository is created with this `.gitignore`. |

**Result** (`MyModCreated`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `pack_path` | `string` | yes | Path of the pack of the MyMod. It doesn't exist until it's saved. |

## `lua.run_tests`

Runs Lua tests against the scripts of the game and the open packs, outside of the game.

Runs as a job: answers right away with a job ID, and sends its result when the job ends (see [Jobs](../ws-protocol.md#jobs)).

**Params** (`RunLuaTests`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `code` | `string` | yes | Code of the Lua test file. |
| `campaign` | `string` or `null` | no | Campaign whose vanilla scripts to load, like `main_warhammer`. If not set, only the script libraries and the mods are loaded. |

**Result** (`LuaTestResults`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `report` | `any` | yes | The report of the tests: passed and failed tests, with their errors and output. |

## `startpos.campaigns`

Returns the campaigns a startpos can be built for.

**Params** (`GetStartposCampaigns`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `pack` | `string` | yes | Key of the pack with the startpos tables. |

**Result** (`StartposCampaigns`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `campaigns` | array of `string` | yes | Campaign keys, sorted. |
| `last_used` | `string` or `null` | yes | Campaign the last startpos of the pack was built for, if any. |

## `startpos.start`

Prepares the Assembly Kit to build a startpos with the tables of an open pack, and launches the game to build it.

When the game is closed, call `startpos.finish` to import the startpos into the pack.

**Params** (`StartStartpos`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `pack` | `string` | yes | Key of the pack with the startpos tables. |
| `campaign` | `string` | yes | Campaign to build the startpos for. |
| `process_hlp_spd_data` | `boolean` | no, default `false` | If the HLP and SPD data are built too. |

**Result** (`Done`)

Empty object: `{}`.

## `startpos.finish`

Imports the startpos built by the game into the pack, or cancels the build, and cleans up the build files.

**Params** (`FinishStartpos`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `cancel` | `boolean` | no, default `false` | If the build is cancelled instead of imported. |

**Result** (`FilesEdited`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `edited` | array of `string` | yes | Paths of the edited files. |

## `startpos.validate`

Checks the tables a campaign's startpos is built from, and reports what would make the build
fail, hang, or drop data. Works for both ways of building startposes.

Runs as a job: answers right away with a job ID, and sends its result when the job ends (see [Jobs](../ws-protocol.md#jobs)).

**Params** (`ValidateStartpos`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `pack` | `string` | yes | Key of the pack with the startpos tables. |
| `campaign` | `string` | yes | Campaign to check. |

**Result** (`StartposValidation`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `issues` | array of [`StartposIssue`](#type-startposissue) | yes | The problems, errors first. |

## `startpos.generate`

Generates the startpos of a campaign from the tables of an open pack and the game's files, and
adds it to the pack as `campaigns/<campaign>/startpos.esf`. Unlike `startpos.start`, it doesn't launch the game.

The tables are validated first (see `startpos.validate`), and the startpos isn't generated if they have errors.

Experimental, and only available for Attila. The record layouts are taken from the game's own startpos of the
campaign (or of one of its campaigns, for new ones) and cached in the config folder.

Runs as a job: answers right away with a job ID, and sends its result when the job ends (see [Jobs](../ws-protocol.md#jobs)).

**Params** (`GenerateStartpos`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `pack` | `string` | yes | Key of the pack with the startpos tables. |
| `campaign` | `string` | yes | Campaign to generate the startpos for. |

**Result** (`FilesEdited`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `edited` | array of `string` | yes | Paths of the edited files. |

## `tools.plugin_scripts`

Returns the plugin scripts in the scripts folder of the config.

**Params** (`ListPluginScripts`)

Empty object: `{}`.

**Result** (`PluginScripts`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `scripts` | array of `string` | yes | Paths of the scripts. |

## `tools.run_plugin_script`

Runs a plugin script over files of an open pack, reading back the files it changes.

**Params** (`RunPluginScript`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `pack` | `string` | yes | Key of the pack. |
| `script` | `string` | yes | Path of the script. |
| `paths` | array of `string` | yes | Paths of the files and folders passed to the script. |

**Result** (`PluginScriptRun`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `edited` | array of `string` | yes | Paths of the files read back into the pack. |
| `error` | `string` or `null` | yes | Error message of the script, if it failed. |

## `lua.hovers`

Returns the docs of the game's Lua API for the symbols of a Lua script, to show on hover.

**Params** (`GetLuaHovers`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `source` | `string` | yes | Source of the script. |

**Result** (`LuaHovers`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `hovers` | array of [`LuaHover`](#type-luahover) | yes | The hovers. |

## `ceo.traits`

Returns the trait CEOs of the Assembly Kit tables.

**Params** (`ListTraitCeos`)

Empty object: `{}`.

**Result** (`TraitCeos`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `traits` | array of [`string`, `string`] | yes | UUID and key of each trait CEO. |

## `ceo.add_entries`

Adds CEO entries (armour, career, traits and their locs) to an open pack.

**Params** (`AddCeoEntries`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `pack` | `string` | yes | Key of the pack. |
| `entries` | array of [`CeoEntryData`](#type-ceoentrydata) | yes | The entries. |

**Result** (`FilesEdited`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `edited` | array of `string` | yes | Paths of the edited files. |

## `ceo.build`

Builds `ceo_data.ccd` in the Assembly Kit from the CEO tables of an open pack, running BOB.

**Params** (`BuildCeo`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `pack` | `string` | yes | Key of the pack. |
| `assembly_kit` | `string` | yes | Root folder of the Assembly Kit. |
| `bob` | `string` | yes | Path of BOB's executable. |

**Result** (`Done`)

Empty object: `{}`.

## `ceo.import`

Imports the `ceo_data.ccd` built by `ceo.build` into an open pack.

**Params** (`ImportCeo`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `pack` | `string` | yes | Key of the pack. |
| `assembly_kit` | `string` | yes | Root folder of the Assembly Kit. |

**Result** (`FilesEdited`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `edited` | array of `string` | yes | Paths of the edited files. |

## Types

### Type `CeoEntryData`

A CEO entry to add.

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `name` | `string` | yes | Name of the entry, used to build the keys of its rows. |
| `option` | `string` | yes | Kind of entry: `unique` for unique ones. |
| `element` | `string` | yes | Element of the entry. |
| `gender` | `string` | yes | Gender of the entry. |
| `traits` | array of [`string`, `string`] | yes | UUID and key of each trait of the entry. |
| `expanded` | `boolean` | yes | If the entry also starts with a sword and shield. |

### Type `FileRef`

A file in one of the sources.

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `source` | [`FileSource`](#type-filesource) | yes | Where the file is. |
| `path` | `string` | yes | Path of the file in its source. |

### Type `FileSource`

Where a file is.

One of:

- `{ "pack": … }`: An open pack, by key.
    - `pack` (`string`, yes)
- `"game_files"`: The vanilla files of the selected game.
- `"parent_files"`: The files of the parent packs of the open packs.
- `"assembly_kit"`: The tables of the selected game's Assembly Kit. Their paths are `db/<table_name>/ak_data`.

### Type `LuaHover`

Docs of a symbol of a Lua script. Lines and columns are 0-based.

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `start_line` | `integer` | yes | Line where the symbol starts. |
| `start_column` | `integer` | yes | Column where the symbol starts. |
| `end_line` | `integer` | yes | Line where the symbol ends. |
| `end_column` | `integer` | yes | Column where the symbol ends. |
| `docs` | `string` | yes | The docs, as rich text. |

### Type `MapTile`

A tile of a map.

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `path` | `string` | yes | Path on disk of the tile. |
| `folder` | `string` | yes | Subfolder of the pack's terrain tiles folder to add it to. |

### Type `StartposIssue`

A problem in the tables a startpos is built from.

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `level` | [`DiagnosticLevel`](#type-diagnosticlevel) | yes | `error` if the build fails or hangs, `warning` if it works but drops or ignores data. |
| `table` | `string` | yes | Table, without the `_tables` suffix. |
| `row` | `string` | yes | Key values of the row, joined by `\|`, or its position (`#n`) if the table has no keys. |
| `column` | `string` or `null` | yes | Column with the problem, if it's about one. |
| `message` | `string` | yes | What's wrong. |

### Type `DiagnosticLevel`

Severity of a result.

One of:

- `"info"`: A suggestion, or information about the mod.
- `"warning"`: Something that may be a mistake, or cause problems in some situations.
- `"error"`: Something that will likely cause problems, like invalid references or malformed data.
