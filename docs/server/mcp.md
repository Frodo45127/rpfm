# MCP Interface

In addition to the [WebSocket protocol](./overview.md), the RPFM Server exposes a **Model Context Protocol (MCP)** interface at `/mcp`. This allows AI assistants (such as Claude, Cursor, or any MCP-compatible client) to interact with RPFM programmatically using the standard [MCP specification](https://modelcontextprotocol.io/).

## Transport

The MCP endpoint uses **Streamable HTTP** transport:

```
POST http://127.0.0.1:45127/mcp
```

Each MCP connection gets its own RPFM session, just like WebSocket connections.

## How It Differs from WebSocket

| Aspect            | WebSocket (`/ws`)                             | MCP (`/mcp`)                                      |
|-------------------|-----------------------------------------------|---------------------------------------------------|
| Protocol          | Custom JSON messages with `id`/`data` envelope | Standard MCP (JSON-RPC 2.0)                       |
| Transport         | WebSocket                                      | Streamable HTTP                                   |
| Interaction model | Send `Command`, receive `Response`             | Call named **tools**, receive JSON results         |
| Session control   | Manual via `?session_id=` and `ClientDisconnecting` | Managed automatically by the MCP transport  |
| Intended clients  | Custom scripts, GUIs                           | AI assistants and MCP-compatible tools             |

Both interfaces expose the same underlying functionality — every MCP tool maps to an internal `Command` and returns its `Response` serialized as JSON.

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

### Other MCP Clients

Any MCP client that supports Streamable HTTP transport can connect. Point it to `http://127.0.0.1:45127/mcp`.

## Tool Reference

The MCP interface exposes **150 tools** organized by category. Each tool accepts typed JSON arguments and returns the server's `Response` serialized as JSON text.

### Generic

| Tool | Description | Arguments |
|------|-------------|-----------|
| `call_command` | Call any IPC command directly (for commands not yet wrapped as named tools) | `command`: JSON string of the Command enum |

### Pack Lifecycle

| Tool | Description | Key Arguments |
|------|-------------|---------------|
| `new_pack` | Create a new empty PackFile | *(none)* |
| `open_packfiles` | Open one or more PackFiles | `paths`: file paths |
| `save_packfile` | Save a pack | `pack_key` |
| `close_pack` | Close a pack without saving | `pack_key` |
| `close_all_packs` | Close every open pack without saving | *(none)* |
| `save_pack_as` | Save a pack to a new path | `pack_key`, `path` |
| `clean_and_save_pack_as` | Save a clean copy (use if normal save fails) | `pack_key`, `path` |
| `trigger_backup_autosave` | Trigger a backup autosave | `pack_key` |
| `load_all_ca_pack_files` | Open all vanilla CA PackFiles for the selected game | *(none)* |
| `list_open_packs` | List all open packs with their keys and metadata | *(none)* |

### Pack Metadata

| Tool | Description | Key Arguments |
|------|-------------|---------------|
| `set_pack_file_type` | Set the pack type (PFHFileType as JSON) | `pack_key`, `pack_file_type` |
| `change_compression_format` | Change compression format | `pack_key`, `format` |
| `change_index_includes_timestamp` | Toggle timestamp in pack index | `pack_key`, `value` |
| `get_pack_file_path` | Get the file path of a pack | `pack_key` |
| `get_pack_file_name` | Get the file name of a pack | `pack_key` |
| `get_pack_settings` | Get pack settings | `pack_key` |
| `set_pack_settings` | Set pack settings (PackSettings as JSON) | `pack_key`, `settings` |
| `get_dependency_pack_files_list` | Get dependency pack list | `pack_key` |
| `set_dependency_pack_files_list` | Set dependency pack list | `pack_key`, `list` |

### File Operations

| Tool | Description | Key Arguments |
|------|-------------|---------------|
| `decode_packed_file` | Decode a file from a pack | `pack_key`, `path`, `source` |
| `new_packed_file` | Create a new file inside a pack | `pack_key`, `path`, `new_file` |
| `add_packed_files` | Add files from disk to a pack | `pack_key`, `source_paths`, `destination_paths` |
| `add_packed_files_from_pack_file` | Add files from another PackFile | `pack_key`, `source_pack_path`, `container_paths` |
| `add_packed_files_from_pack_file_to_animpack` | Add files to an AnimPack | `pack_key`, `animpack_path`, `container_paths` |
| `add_packed_files_from_animpack` | Add files from an AnimPack | `pack_key`, `source`, `animpack_path`, `container_paths` |
| `delete_packed_files` | Delete files from a pack | `pack_key`, `paths` |
| `copy_packed_files` | Copy paths into the internal clipboard | `paths_by_pack` |
| `cut_packed_files` | Cut paths into the internal clipboard (removed from source on paste) | `paths_by_pack` |
| `paste_packed_files` | Paste from the internal clipboard into a pack folder | `pack_key`, `destination_path` |
| `duplicate_packed_files` | Duplicate files in-place within the same pack (numeric suffix added) | `pack_key`, `paths` |
| `delete_from_animpack` | Delete files from an AnimPack | `pack_key`, `animpack_path`, `container_paths` |
| `extract_packed_files` | Extract files to disk | `pack_key`, `source_paths`, `destination_path`, `export_as_tsv` |
| `rename_packed_files` | Rename files in a pack | `pack_key`, `renames` |
| `save_packed_file_from_view` | Save an edited decoded file back | `pack_key`, `path`, `data` |
| `save_packed_file_from_external_view` | Save a file from an external program | `pack_key`, `internal_path`, `external_path` |
| `save_packed_files_to_pack_file_and_clean` | Save files and optionally optimize | `pack_key`, `files`, `optimize` |
| `get_packed_file_raw_data` | Get raw binary data of a file | `pack_key`, `value` |
| `open_packed_file_in_external_program` | Open a file in an external program | `pack_key`, `source`, `container_path` |
| `open_containing_folder` | Open the pack's folder in file manager | `pack_key` |
| `clean_cache` | Clean the decode cache | `pack_key`, `paths` |
| `folder_exists` | Check if a folder exists in a pack | `pack_key`, `value` |
| `packed_file_exists` | Check if a file exists in a pack | `pack_key`, `value` |
| `get_packed_files_info` | Get info of one or more files | `pack_key`, `values` |
| `get_rfile_info` | Get info of a single file | `pack_key`, `value` |

### Game Selection

| Tool | Description | Key Arguments |
|------|-------------|---------------|
| `get_game_selected` | Get the currently selected game | *(none)* |
| `set_game_selected` | Set the current game | `game_name`, `rebuild_dependencies` |

### Dependencies

| Tool | Description | Key Arguments |
|------|-------------|---------------|
| `generate_dependencies_cache` | Generate the dependencies cache | *(none)* |
| `rebuild_dependencies` | Rebuild dependencies | `value` (true = full) |
| `is_there_a_dependency_database` | Check if a dependency database is loaded | `value` |
| `get_table_list_from_dependency_pack_file` | Get table names from dependency packs | *(none)* |
| `get_custom_table_list` | Get custom table names from schema | *(none)* |
| `get_table_version_from_dependency_pack_file` | Get table version from dependencies | `value` |
| `get_table_definition_from_dependency_pack_file` | Get table definition from dependencies | `value` |
| `get_tables_from_dependencies` | Get table data by name | `value` |
| `import_dependencies_to_open_pack_file` | Import files from dependencies | `pack_key`, `paths` |
| `get_rfiles_from_all_sources` | Get files from all sources | `paths`, `lowercase` |
| `get_packed_files_names_starting_with_path_from_all_sources` | Get file names under a path | `path` |
| `local_art_set_ids` | Get local art set IDs | `pack_key` |
| `dependencies_art_set_ids` | Get art set IDs from dependencies | *(none)* |

### Search

| Tool | Description | Key Arguments |
|------|-------------|---------------|
| `global_search` | Run a global search | `pack_key`, `search` |
| `global_search_replace_matches` | Replace specific matches | `pack_key`, `search`, `matches` |
| `global_search_replace_all` | Replace all matches | `pack_key`, `search` |
| `search_references` | Find all references to a value | `pack_key`, `reference_map`, `value` |
| `get_reference_data_from_definition` | Get reference data for columns | `pack_key`, `table_name`, `definition`, `force` |
| `go_to_definition` | Go to a reference's definition | `pack_key`, `table_name`, `column_name`, `values` |
| `go_to_loc` | Go to a loc key's location | `pack_key`, `value` |
| `get_source_data_from_loc_key` | Get source data of a loc key | `pack_key`, `value` |

### Schema

| Tool | Description | Key Arguments |
|------|-------------|---------------|
| `save_schema` | Save a schema to disk | `schema` |
| `update_current_schema_from_asskit` | Update schema from Assembly Kit | *(none)* |
| `update_schemas` | Update schemas from remote repository | *(none)* |
| `is_schema_loaded` | Check if a schema is loaded | *(none)* |
| `get_schema` | Get the current schema | *(none)* |
| `definitions_by_table_name` | Get definitions for a table | `value` |
| `definition_by_table_name_and_version` | Get a definition by name and version | `name`, `version` |
| `delete_definition` | Delete a definition | `name`, `version` |
| `referencing_columns_for_definition` | Get columns referencing a table | `table_name`, `definition` |
| `fields_processed` | Get processed fields from a definition | `definition` |
| `save_local_schema_patch` | Save local schema patches | `patches` |
| `remove_local_schema_patches_for_table` | Remove patches for a table | `value` |
| `remove_local_schema_patches_for_table_and_field` | Remove patches for a field | `key`, `value` |
| `import_schema_patch` | Import a schema patch | `patches` |

### Table Operations

| Tool | Description | Key Arguments |
|------|-------------|---------------|
| `merge_files` | Merge compatible tables into one | `pack_key`, `paths`, `merged_path`, `delete_source` |
| `update_table` | Update a table to a newer version | `pack_key`, `value` |
| `cascade_edition` | Cascade edit across referenced data | `pack_key`, `table_name`, `definition`, `changes` |
| `get_tables_by_table_name` | Get table paths by name | `pack_key`, `value` |
| `add_keys_to_key_deletes` | Add keys to key_deletes table | `pack_key`, `table_file_name`, `key_table_name`, `keys` |
| `export_tsv` | Export a table to TSV | `pack_key`, `tsv_path`, `table_path` |
| `import_tsv` | Import a TSV file to a table | `pack_key`, `tsv_path`, `table_path` |

### Diagnostics

| Tool | Description | Key Arguments |
|------|-------------|---------------|
| `diagnostics_check` | Run a full diagnostics check | `ignored`, `check_ak_only_refs` |
| `diagnostics_update` | Update diagnostics for changed files | `diagnostics`, `paths`, `check_ak_only_refs` |
| `add_line_to_pack_ignored_diagnostics` | Add to ignored diagnostics | `pack_key`, `value` |
| `get_missing_definitions` | Export missing table definitions | `pack_key` |

### Notes

| Tool | Description | Key Arguments |
|------|-------------|---------------|
| `notes_for_path` | Get notes under a path | `pack_key`, `value` |
| `add_note` | Add a note | `pack_key`, `note` |
| `delete_note` | Delete a note | `pack_key`, `path`, `id` |

### Optimization

| Tool | Description | Key Arguments |
|------|-------------|---------------|
| `optimize_pack_file` | Optimize a pack | `pack_key`, `options` |
| `get_optimizer_options` | Get default optimizer options | *(none)* |

### Updates

| Tool | Description |
|------|-------------|
| `check_updates` | Check for RPFM updates |
| `check_schema_updates` | Check for schema updates |
| `check_lua_autogen_updates` | Check for Lua autogen updates |
| `check_empire_and_napoleon_ak_updates` | Check for Empire/Napoleon AK updates |
| `check_translations_updates` | Check for translation updates |
| `update_lua_autogen` | Update Lua autogen |
| `update_main_program` | Update the program |
| `update_empire_and_napoleon_ak` | Update Empire/Napoleon AK files |
| `update_translations` | Update translations |

### Settings

| Tool | Description | Key Arguments |
|------|-------------|---------------|
| `settings_get_bool` | Get a boolean setting | `value` (key) |
| `settings_get_i32` | Get an i32 setting | `value` (key) |
| `settings_get_f32` | Get an f32 setting | `value` (key) |
| `settings_get_string` | Get a string setting | `value` (key) |
| `settings_get_path_buf` | Get a PathBuf setting | `value` (key) |
| `settings_get_vec_string` | Get a Vec\<String\> setting | `value` (key) |
| `settings_get_vec_raw` | Get a raw bytes setting | `value` (key) |
| `settings_get_all` | Get all settings at once | *(none)* |
| `settings_set_bool` | Set a boolean setting | `key`, `value` |
| `settings_set_i32` | Set an i32 setting | `key`, `value` |
| `settings_set_f32` | Set an f32 setting | `key`, `value` |
| `settings_set_string` | Set a string setting | `key`, `value` |
| `settings_set_path_buf` | Set a PathBuf setting | `key`, `value` |
| `settings_set_vec_string` | Set a Vec\<String\> setting | `key`, `value` |
| `settings_set_vec_raw` | Set a raw bytes setting | `key`, `value` |
| `backup_settings` | Backup settings to memory | *(none)* |
| `clear_settings` | Clear all settings | *(none)* |
| `restore_backup_settings` | Restore settings from backup | *(none)* |

### Path Queries

| Tool | Description |
|------|-------------|
| `config_path` | Get the config path |
| `assembly_kit_path` | Get the Assembly Kit path |
| `backup_autosave_path` | Get the backup autosave path |
| `old_ak_data_path` | Get the old AK data path |
| `schemas_path` | Get the schemas path |
| `table_profiles_path` | Get the table profiles path |
| `translations_local_path` | Get the translations local path |
| `dependencies_cache_path` | Get the dependencies cache path |
| `settings_clear_path` | Clear a config path |

### Specialized

| Tool | Description | Key Arguments |
|------|-------------|---------------|
| `open_pack_info` | Get pack info and file list | `pack_key` |
| `initialize_my_mod_folder` | Initialize a MyMod folder | `name`, `game`, `sublime`, `vscode`, `gitignore` |
| `live_export` | Live export a pack for testing | `pack_key` |
| `patch_siege_ai` | Patch SiegeAI for Warhammer maps | `pack_key` |
| `pack_map` | Pack map tiles | `pack_key`, `tile_maps`, `tiles` |
| `generate_missing_loc_data` | Generate missing loc entries | `pack_key` |
| `get_pack_translation` | Get translation data | `pack_key`, `language` |
| `build_starpos_get_campaign_ids` | Get campaign IDs for starpos | `pack_key` |
| `build_starpos_check_victory_conditions` | Check victory conditions file | `pack_key` |
| `build_starpos` | Build starpos (pre-processing) | `pack_key`, `campaign_id`, `process_hlp_spd` |
| `build_starpos_post` | Build starpos (post-processing) | `pack_key`, `campaign_id`, `process_hlp_spd` |
| `build_starpos_cleanup` | Clean up starpos temp files | `pack_key`, `campaign_id`, `process_hlp_spd` |
| `update_anim_ids` | Update animation IDs | `pack_key`, `starting_id`, `offset` |
| `get_anim_paths_by_skeleton_name` | Get anim paths by skeleton | `value` |
| `export_rigid_to_gltf` | Export RigidModel to glTF | `rigid_model`, `output_path` |
| `set_video_format` | Change video format | `pack_key`, `path`, `format` |

## Response Format

All MCP tool responses are JSON-serialized versions of the server's internal `Response` enum. The same [serialization convention](./overview.md#serialization-convention) applies — refer to the [Responses](./ws-responses.md) page for the full list of response types and their payloads.

## Typical Workflow

A typical MCP session follows this pattern:

1. **Set the game**: `set_game_selected` with the game key and `rebuild_dependencies: true`
2. **Open a pack**: `open_packfiles` with the file path(s)
3. **Browse files**: `open_pack_info` to get the pack's file tree
4. **Read data**: `decode_packed_file` to decode individual files
5. **Modify data**: `save_packed_file_from_view` to save edited data back
6. **Save**: `save_packfile` to write changes to disk

### Example: Reading a DB Table

Call `set_game_selected`:
```json
{ "game_name": "warhammer_3", "rebuild_dependencies": true }
```

Call `open_packfiles`:
```json
{ "paths": ["/path/to/my_mod.pack"] }
```

The response contains the `pack_key` — use it in subsequent calls:

Call `decode_packed_file`:
```json
{ "pack_key": "the_pack_key", "path": "db/units_tables/data", "source": "PackFile" }
```

The response will be a `DBRFileInfo` containing the decoded table data and file metadata.

### Common `pack_key` Pattern

Most tools require a `pack_key` argument to identify which open pack to operate on. After opening a pack with `open_packfiles`, the response includes the key. You can also call `list_open_packs` at any time to see all open packs and their keys.

### JSON String Arguments

Some tools accept complex types (like `ContainerPath`, `Definition`, `GlobalSearch`) as JSON strings. These must be passed as a **serialized JSON string** in the argument field — not as a nested object. For example:

```json
{
  "pack_key": "my_pack",
  "paths": "[{\"File\": \"db/units_tables/data\"}]"
}
```

Refer to the [Shared Types](./ws-shared-types.md) page for the structure of these types.
