# MCP Interface

In addition to the [WebSocket protocol](./ws-protocol.md), the RPFM Server exposes a **Model Context Protocol (MCP)** interface at `/mcp`. This allows AI assistants (such as Claude, Cursor, or any MCP-compatible client) to interact with RPFM using the standard [MCP specification](https://modelcontextprotocol.io/).

## Transport

The MCP endpoint uses **Streamable HTTP** transport:

```
POST http://127.0.0.1:45127/mcp
```

Each MCP connection gets its own RPFM session, separate from the WebSocket ones. MCP sessions are reaped by the server after 5 minutes of inactivity, because the MCP transport gives no reliable disconnect signal.

MCP sessions start with the settings in RPFM's `settings.json`, and can't change them.

## How It Differs from WebSocket

| Aspect            | WebSocket (`/ws`)                                | MCP (`/mcp`)                                        |
|-------------------|--------------------------------------------------|-----------------------------------------------------|
| Protocol          | JSON-RPC 2.0                                     | MCP (JSON-RPC 2.0)                                  |
| Transport         | WebSocket                                        | Streamable HTTP                                     |
| Interaction model | Call [methods](./methods.md)                     | Call named **tools**                                |
| Slow operations   | Return a job ID, followed by `job.updated` notifications | Wait up to 45 seconds for the job, then return its state |
| Session control   | `?session_id=` and `session.disconnect`          | Reaped after 5 minutes of inactivity                |
| Intended clients  | Custom scripts, GUIs                             | AI assistants and MCP-compatible tools              |

Every tool calls one method, with the same params and result, so the [Methods](./methods.md) reference also documents the tools. The tools only cover what's useful to an assistant: methods made for the UI, like the ones returning whole decoded files, have no tool.

## Connecting

### Claude Desktop

Add the server to your `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "rpfm": {
      "url": "http://127.0.0.1:45127/mcp"
    }
  }
}
```

### Claude Code

```bash
claude mcp add --transport http rpfm http://127.0.0.1:45127/mcp
```

### Other MCP Clients

Any MCP client that supports Streamable HTTP transport can connect. Point it to `http://127.0.0.1:45127/mcp`.

## Results and Errors

Tools return their result as **structured content**, and their params and results are described by JSON schemas in `tools/list`.

A tool that fails returns a **tool error** instead of ending the MCP session. Its content has the same `code`, `message` and `data.kind` as a [WebSocket error](./ws-protocol.md#errors), like `pack_not_found` or `schema_not_loaded`.

### Jobs

Slow tools (`set_game`, `generate_dependencies_cache`, `rebuild_dependencies`, `run_diagnostics`, `run_search`, `update_schemas`, `update_schema_from_assembly_kit`, `optimize_pack`, `run_lua_tests`) run as [jobs](./ws-protocol.md#jobs). They wait up to 45 seconds and return the state of their job:

- If it finished, the state includes the `result` of the tool.
- If it failed, the tool returns a tool error with the state.
- If it's still running, call `wait_for_job` with its `job` ID, as many times as needed.

Other tools called meanwhile wait for the job to end before running, except during `run_diagnostics`: a running check stops for them, and starts again after them.

## Resources

The server also exposes reference data as MCP resources, readable without tool calls:

| URI | Content |
|-----|---------|
| `rpfm://games` | Keys of the supported games. |
| `rpfm://enums/PFHFileType` | Valid pack types. |
| `rpfm://enums/CompressionFormat` | Valid compression formats. |
| `rpfm://enums/SupportedFormats` | Valid video formats. |
| `rpfm://reference/initialization` | How to set up a session. |
| `rpfm://reference/path_conventions` | Common file paths inside packs. |

## Typical Workflow

1. **Set the game**: `set_game` with the game key. This loads the schema and the dependencies.
2. **Open a pack**: `open_pack` with the path of the pack, or `new_pack`. Both return the pack's `key`.
3. **Browse files**: `list_files` with the pack as source.
4. **Read data**: `table_info` and `table_rows` for DB and Loc tables, `read_file` for anything else.
5. **Modify data**: `edit_table` for tables, `write_file` for anything else.
6. **Save**: `save_pack`.

### Example: Editing a DB Table

Call `set_game`:
```json
{ "game": "warhammer_3" }
```

Call `open_pack`:
```json
{ "paths": ["/path/to/my_mod.pack"] }
```

The result contains the pack's `key`, used by the next calls as `pack`. `session_status` lists the keys of all open packs.

Call `table_rows`, to get the rows of a unit:
```json
{
  "file": { "source": { "pack": "my_mod.pack" }, "path": "db/land_units_tables/my_mod" },
  "columns": ["key", "num_men"],
  "filters": [{ "column": "key", "op": "equals", "value": "my_unit" }]
}
```

Each returned row has the `index` of the row in the table. Call `edit_table` to change it:
```json
{
  "file": { "source": { "pack": "my_mod.pack" }, "path": "db/land_units_tables/my_mod" },
  "edits": [{ "op": "update", "index": 4, "values": { "num_men": 120 } }]
}
```

Call `save_pack`:
```json
{ "pack": "my_mod.pack" }
```

To change a vanilla table, copy it into your pack first with `copy_files`, from the `"game_files"` source.

## Tool Reference

The MCP interface exposes **88 tools**. The descriptions below are the first sentence of each tool's description; `tools/list` has the full descriptions and the JSON schemas of their params.

### Session and jobs

| Tool | Method | Description |
|------|--------|-------------|
| `session_status` | `session.status` | Get the state of the session: the selected game, if its schema and dependencies (vanilla files, Assembly Kit tables, parent packs) are loaded, and the open packs with their keys. |
| `set_game` | `session.set_game` | Select the game to work with, like `warhammer_3`, loading its schema and, by default, its dependencies (vanilla files, Assembly Kit tables, parent packs). |
| `job_status` | `job.status` | Get the state of a job: queued, running (with its current step), finished (with its result), failed (with its error) or cancelled. |
| `wait_for_job` | `job.wait` | Wait for a job to end, up to `timeout_secs` (60 by default), and return its state. |
| `cancel_job` | `job.cancel` | Cancel a job that hasn't started yet. |

### Packs

| Tool | Method | Description |
|------|--------|-------------|
| `pack_info` | `pack.info` | Get the details of an open pack: type, format version, compression, encryption flags, the packs it depends on, and its MyMod mode. |
| `pack_settings` | `pack.settings` | Get the settings of an open pack, like its diagnostics ignore rules (`diagnostics_files_to_ignore`), files to ignore when importing, or if it has autosaves disabled. |
| `update_pack_settings` | `pack.update_settings` | Change settings of an open pack. |
| `new_pack` | `pack.new` | Create a new empty pack. |
| `open_pack` | `pack.open` | Open one or more packs from disk, merged into a single one. |
| `open_vanilla_packs` | `pack.open_vanilla` | Open all the vanilla packs of the selected game, merged into a single one, to browse them like any open pack. |
| `close_pack` | `pack.close` | Close an open pack. |
| `close_all_packs` | `pack.close_all` | Close all open packs. |
| `save_pack` | `pack.save` | Save an open pack to disk, to its current path or to a new `path`. |
| `update_pack` | `pack.update` | Change properties of an open pack: type, compression, encryption and timestamp flags, the packs it depends on, and its MyMod mode. |

### Files

| Tool | Method | Description |
|------|--------|-------------|
| `list_files` | `files.list` | List the files of an open pack, the game files, the parent packs, or the Assembly Kit tables, sorted by path. |
| `read_file` | `file.read` | Read a file of an open pack, the game files or the parent packs: as `text` for scripts, XML, JSON and other text files; as `decoded` JSON for structured formats (portrait settings, unit variants, models, etc.); or as `raw` base64 bytes, best for images and other binary files. |
| `write_file` | `file.write` | Replace the contents of a file of an open pack: `text` for text files, `decoded` JSON in the format `read_file` returns, or `raw` base64 bytes (which also creates the file if it doesn't exist). |
| `list_animpack` | `animpack.list` | List the files inside an AnimPack of an open pack, the game files or the parent packs. |
| `add_to_animpack` | `animpack.add` | Copy files of an open pack into an AnimPack of an open pack. |
| `extract_from_animpack` | `animpack.extract` | Copy files of an AnimPack of any source into an open pack. |
| `delete_from_animpack` | `animpack.delete` | Delete files from an AnimPack of an open pack. |
| `create_file` | `files.create` | Create a new empty file in an open pack: a DB table (with the version of the game files by default), a Loc table, a text file or an AnimPack. |
| `add_files_from_disk` | `files.add_from_disk` | Add files and folders from disk to an open pack, under a folder of the pack. |
| `copy_files` | `files.copy` | Copy files and folders from an open pack, the game files, the parent packs or the Assembly Kit tables into an open pack, keeping their paths. |
| `delete_files` | `files.delete` | Delete files and folders from an open pack. |
| `rename_files` | `files.rename` | Rename or move files and folders of an open pack. |
| `duplicate_files` | `files.duplicate` | Copy files of an open pack in the same pack, adding a number to their names. |
| `extract_files` | `files.extract` | Extract files and folders of an open pack, the game files or the parent packs to a folder on disk, optionally with tables as TSV. |

### Tables

| Tool | Method | Description |
|------|--------|-------------|
| `table_info` | `table.info` | Get the columns (name, type, key, referenced table and column, default value, description) and the row count of a DB or Loc table, from any source. |
| `table_rows` | `table.rows` | Read rows of a DB or Loc table, from any source, as plain values (booleans, numbers and strings). |
| `column_values` | `table.column_values` | Get the distinct values of a column of a table, like all the faction keys of `factions_tables`, from the open packs and the dependencies, sorted. |
| `merge_tables` | `table.merge` | Merge tables of the same type of an open pack into a new one. |
| `upgrade_table` | `table.upgrade` | Update a table of an open pack to the version it has in the game files, after a game update. |
| `rename_key` | `table.rename_key` | Change a key value of a table in every table of an open pack: the key itself, the columns referencing it, and the loc keys generated from it. |
| `add_key_deletes` | `table.add_key_deletes` | Add keys to a key deletes table (`db/twad_key_deletes_tables/<file_name>`) of an open pack, to delete those keys of a table in the game. |
| `export_tsv` | `table.export_tsv` | Write a table of an open pack, the game files or the parent packs to a TSV file, to edit it in a spreadsheet. |
| `import_tsv` | `table.import_tsv` | Replace a table of an open pack with the contents of a TSV file, keeping its GUID. |
| `edit_table` | `table.edit` | Edit rows of a DB or Loc table in an open pack: insert, update and delete rows by index, with values given by column name (see `table_info` for the columns). |

### Schema

| Tool | Method | Description |
|------|--------|-------------|
| `table_definition` | `schema.definition` | Get the columns of a table as defined in the schema, without needing a file of it. |
| `schema_tables` | `schema.tables` | List the tables of the selected game's schema whose name starts with a prefix, with the versions it has definitions for, newest first. |
| `patch_column` | `schema.patch_column` | Change how the schema describes a column with a local patch: description, if it's a key, default value, referenced table and column (`is_reference` as `table;column`), if it holds file paths, if it can't be empty, or if it's unused. |
| `remove_patches` | `schema.remove_patches` | Remove the local schema patches of a table, or of one of its columns. |
| `update_schemas` | `schema.update` | Download the latest schemas, reload the selected game's one, and rebuild the dependencies. |
| `update_schema_from_assembly_kit` | `schema.update_from_assembly_kit` | Update the selected game's schema with the table definitions of its Assembly Kit, and save it. |
| `raw_definitions` | `schema.raw_definitions` | Get the definitions of a table as the schema stores them, to edit them with `set_definition`. |
| `set_definition` | `schema.set_definition` | Add a definition to the selected game's schema, or replace the one with its version, then save and reload the schema. |
| `delete_definition` | `schema.delete_definition` | Remove a definition from the selected game's schema, then save and reload the schema. |
| `referencing_columns` | `schema.referencing_columns` | Get the columns of other tables referencing each column of a table, according to the schema. |
| `table_patches` | `schema.patches` | Get the patches applied to the columns of a table definition: the local ones made with `patch_column`, and the ones included in the schema. |
| `missing_definitions` | `schema.missing_definitions` | List the tables of an open pack with rows the schema can't decode, which need a new definition. |
| `import_patches` | `schema.import_patches` | Add patches to the selected game's schema itself, and save it. |

### Dependencies

| Tool | Method | Description |
|------|--------|-------------|
| `generate_dependencies_cache` | `dependencies.generate_cache` | Generate the dependencies cache of the selected game from its files and Assembly Kit, and load it. |
| `rebuild_dependencies` | `dependencies.rebuild` | Reload the dependencies of the selected game, like after changing the packs the open packs depend on. |
| `dependency_tables` | `dependencies.tables` | List the tables of the selected game's files, and the startpos and twad tables of its schema, with the version new tables of each type should use. |

### References

| Tool | Method | Description |
|------|--------|-------------|
| `find_definition` | `references.definition` | Find the row where a value of a referenced table is defined, like the row of `factions_tables` with a faction key. |
| `find_usages` | `references.usages` | Find the rows of other tables referencing a value of a table, like everything using a faction key. |
| `find_loc` | `references.loc` | Find the row of a Loc file with a key. |
| `loc_source` | `references.loc_source` | Get the table, localised column and key values a loc key belongs to, like `factions`, `screen_name` and `["wh_main_emp_empire"]` for `factions_screen_name_wh_main_emp_empire`. |
| `reference_values` | `references.values` | Get the values a reference column of a table can have, with their display text (like the name of each referenced row), from the open packs and the dependencies. |

### Diagnostics and search

| Tool | Method | Description |
|------|--------|-------------|
| `run_diagnostics` | `diagnostics.run` | Check the open packs for problems (invalid references, duplicated keys, outdated tables, script errors, etc.), and return a summary by level and type. |
| `list_diagnostics` | `diagnostics.list` | List results of the last `run_diagnostics`, filtered by level, report type, pack and path prefix, in pages (100 by default; the total of matching results is always returned). |
| `ignore_diagnostics` | `diagnostics.ignore` | Make the next diagnostics checks of a pack skip results of files under a path, optionally only for some columns and report types. |
| `run_search` | `search.run` | Search text (or a regex) in open packs, the game files, the parent packs, the Assembly Kit tables, or the schema's column names, and return a summary of the matches by file type. |
| `list_search_matches` | `search.matches` | List matches of the last `run_search`, filtered by file type and path prefix, in pages (100 by default; the total of matching matches is always returned). |
| `replace_search_matches` | `search.replace` | Replace matches of the last `run_search` by ID, or all of them, with a text. |
| `run_lua_tests` | `lua.run_tests` | Run Lua tests against the scripts of all open packs, outside of the game. |

### Notes

| Tool | Method | Description |
|------|--------|-------------|
| `list_notes` | `notes.list` | Get the notes (comments) attached to a file or folder of an open pack, or all of them. |
| `add_note` | `notes.add` | Attach a note (comment, with an optional link) to a file or folder of an open pack, or replace one by passing its `id`. |
| `delete_note` | `notes.delete` | Delete a note of an open pack. |

### Translations

| Tool | Method | Description |
|------|--------|-------------|
| `list_translations` | `translations.list` | Get the translation of the texts of an open pack to a language, reusing vanilla and previous translations. |
| `generate_vanilla_texts` | `translations.generate_vanilla` | Generate the vanilla texts of a language from the game's locale packs, so translations to and from it can reuse them. |

### Tools

| Tool | Method | Description |
|------|--------|-------------|
| `optimizer_options` | `tools.optimizer_options` | Get the optimizer options as the server's settings have them, by name. |
| `patch_siege_ai` | `tools.patch_siege_ai` | Patch the siege maps of an open pack so the AI can use them. |
| `pack_map` | `tools.pack_map` | Add the tiles and tile maps of a map exported by Terry to an open pack. |
| `generate_missing_locs` | `tools.generate_missing_locs` | Add empty loc entries for the localised columns of the tables of the open packs that don't have them yet. |
| `update_anim_ids` | `tools.update_anim_ids` | Offset the animation ids of an open pack from a starting id, like after a game update moves them. |
| `anims_by_skeleton` | `tools.anims_by_skeleton` | Get the paths of the animations using a skeleton, in the open packs and the dependencies. |
| `export_gltf` | `tools.export_gltf` | Export a RigidModel of any source to a glTF file, with its textures. |
| `set_video_format` | `tools.set_video_format` | Change the format of a ca_vp8 video of an open pack: `CaVp8` or `Ivf`. |
| `live_export` | `tools.live_export` | Export the scripts and UI files of an open pack to the game's data folder, to test them without saving the pack. |
| `init_mymod` | `tools.init_mymod` | Create the folder of a new MyMod in the MyMods folder of the settings, with optional editor configs for Lua scripting and a git repository. |
| `startpos_campaigns` | `startpos.campaigns` | Get the campaigns a startpos can be built for, and the one the pack's last startpos was built for. |
| `start_startpos` | `startpos.start` | Start building a startpos for a campaign with the tables of an open pack: prepares the Assembly Kit and launches the game. |
| `finish_startpos` | `startpos.finish` | Finish building a startpos after the game was closed: imports it into the pack, or cancels the build with `cancel: true`. |
| `optimize_pack` | `tools.optimize` | Remove data of an open pack that's identical to vanilla or unneeded (duplicated rows, unchanged rows and files, empty tables, etc.). |

### Updates

| Tool | Method | Description |
|------|--------|-------------|
| `check_update` | `updates.check` | Check if there is an update of RPFM (`program`), the `schemas`, the Lua type definitions (`lua_autogen`), the Empire and Napoleon Assembly Kit data (`old_assembly_kit`), or the community `translations`. |
| `apply_update` | `updates.apply` | Download the update of the Lua type definitions (`lua_autogen`), the Empire and Napoleon Assembly Kit data (`old_assembly_kit`), the community `translations`, or RPFM itself (`program`, which replaces its files and needs a restart: only do it if the user asks). |
