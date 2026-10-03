# Methods

Every method of the API, grouped by domain. Each one is a request struct of `rpfm_ipc::api`, whose docs describe every field of the request and of its response: run `cargo doc -p rpfm_ipc --open` in the repository to browse them. Methods marked as jobs answer right away with a job ID, and send their response when the job ends (see [Jobs](./ws-protocol.md#jobs)).

Methods meant for clients with their own views of the files, like the UI, return RPFM's own types (decoded files, typed search matches, full diagnostics). Simpler clients should prefer the paginated methods next to them, like `table.rows`, `search.matches` or `diagnostics.list`.

## Session and dependencies

| Method | Request | Response | Description |
|--------|---------|----------|-------------|
| `session.status` | `GetSessionStatus` | `SessionStatus` | Returns the selected game, what's loaded for it, and the open packs. |
| `session.configure` | `Configure` | `Done` | Replaces the settings the session runs with. |
| `session.disconnect` | `Disconnect` | `Done` | Tells the server the client is closing, so its WebSocket session is removed right away instead of waiting for the timeout. |
| `session.set_game` | `SetGame` | `SessionStatus` | Selects a game, loading its schema and, optionally, its dependencies. Runs as a job. |
| `dependencies.generate_cache` | `GenerateDependenciesCache` | `SessionStatus` | Generates the dependencies cache of the selected game from its files and Assembly Kit, and loads it. Runs as a job. |
| `dependencies.rebuild` | `RebuildDependencies` | `SessionStatus` | Reloads the dependencies of the selected game, like after changing the packs the open packs depend on. Runs as a job. |
| `dependencies.tables` | `ListDependencyTables` | `DependencyTables` | Returns the tables of the game files, and the startpos, twad and CEO tables of the schema, with their version. |
| `dependencies.info` | `GetDependenciesInfo` | `DependenciesInfo` | Returns the files of the dependencies of the selected game. |
| `dependencies.table_data` | `GetDependencyTableData` | `DependencyTableData` | Returns every decoded table of a type in the game files and the parent packs. |

## Packs

| Method | Request | Response | Description |
|--------|---------|----------|-------------|
| `pack.info` | `GetPackInfo` | `PackDetails` | Returns the details of an open pack. |
| `pack.backup` | `BackupPack` | `Done` | Saves a backup copy of an open pack in the autosave folder, removing the oldest copies over the limit in the settings. |
| `pack.new` | `NewPack` | `PackSummary` | Creates a new empty pack. |
| `pack.open` | `OpenPack` | `PackSummary` | Opens one or more packs from disk, merged into a single one. |
| `pack.open_vanilla` | `OpenVanillaPacks` | `PackSummary` | Opens all the vanilla packs of the selected game, merged into a single one. |
| `pack.close` | `ClosePack` | `Done` | Closes an open pack, discarding unsaved changes. |
| `pack.close_all` | `CloseAllPacks` | `Done` | Closes all open packs, discarding unsaved changes. |
| `pack.save` | `SavePack` | `PackSummary` | Saves an open pack to disk. |
| `pack.update` | `UpdatePack` | `PackDetails` | Changes properties of an open pack. |
| `pack.settings` | `GetPackSettings` | `PackSettingsValues` | Returns the settings of an open pack. |
| `pack.update_settings` | `UpdatePackSettings` | `PackSettingsValues` | Changes settings of an open pack. |

## Files

| Method | Request | Response | Description |
|--------|---------|----------|-------------|
| `files.list` | `ListFiles` | `FileList` | Lists the files of a source, sorted by path. |
| `files.create` | `CreateFile` | `FileEntry` | Creates a new empty file in an open pack. |
| `files.add_from_disk` | `AddFilesFromDisk` | `FilesAdded` | Adds files and folders from disk to an open pack. |
| `files.copy` | `CopyFiles` | `FilesAdded` | Copies files and folders from any source into an open pack, keeping their paths. |
| `files.delete` | `DeleteFiles` | `FilesDeleted` | Deletes files and folders from an open pack. |
| `files.rename` | `RenameFiles` | `FilesRenamed` | Renames or moves files and folders of an open pack. |
| `files.duplicate` | `DuplicateFiles` | `FilesAdded` | Copies files of an open pack in the same pack, adding a number to their names. |
| `files.extract` | `ExtractFiles` | `FilesExtracted` | Extracts files and folders of an open pack, the game files or the parent packs to disk. |
| `file.read` | `ReadFile` | `FileContents` | Returns the contents of a file of any source. |
| `file.write` | `WriteFile` | `Done` | Replaces the contents of a file of an open pack. |
| `animpack.list` | `ListAnimPack` | `FileList` | Lists the files inside an AnimPack of any source. |
| `animpack.add` | `AddToAnimPack` | `FilesAdded` | Copies files of an open pack into an AnimPack of an open pack. |
| `animpack.extract` | `ExtractFromAnimPack` | `FilesAdded` | Copies files of an AnimPack of any source into an open pack. |
| `animpack.delete` | `DeleteFromAnimPack` | `Done` | Deletes files from an AnimPack of an open pack. |
| `file.view_data` | `GetViewData` | `ViewData` | Returns a file decoded as a client opening it in a view needs it. |
| `files.info` | `GetFilesInfo` | `FilesInfo` | Returns the info of files of an open pack, like their original pack and last modification time. |
| `files.paste` | `PasteFiles` | `FilesPasted` | Copies or moves files and folders of open packs into a folder of an open pack. |
| `files.from_all_sources` | `GetFilesFromAllSources` | `FilesFromAllSources` | Returns files and folders found in the open packs, the parent packs and the game files. |
| `files.save_files` | `SaveFiles` | `FilesChanged` | Adds files to an open pack, replacing the ones with the same path, optionally optimizing the pack after it. |
| `file.open_external` | `OpenInExternalProgram` | `ExternalFile` | Extracts a file of an open pack to a temporary folder, and opens it in the system's default program for its type. |
| `file.save_external` | `SaveExternalFile` | `Done` | Replaces a file of an open pack with the file on disk an external program edited. |

## Tables

| Method | Request | Response | Description |
|--------|---------|----------|-------------|
| `table.info` | `GetTableInfo` | `TableInfo` | Returns the definition and row count of a table. |
| `table.rows` | `GetTableRows` | `TableRows` | Returns rows of a table, optionally filtered and with only some columns. |
| `schema.definition` | `GetTableDefinition` | `TableDefinition` | Returns the columns of a table as defined in the schema. |
| `table.edit` | `EditTable` | `TableEdited` | Edits rows of a DB or Loc table in an open pack. |
| `table.column_values` | `GetColumnValues` | `ColumnValues` | Returns the distinct values of a column of a table, sorted. |
| `table.unused_numbers` | `GetUnusedNumbers` | `UnusedNumbers` | Returns numbers no row uses yet in an integer column of a table, for new rows needing unique ones, like ids. |
| `table.merge` | `MergeTables` | `TablesMerged` | Merges tables of the same type of an open pack into a new one. |
| `table.upgrade` | `UpgradeTable` | `TableUpgraded` | Updates a table of an open pack to the version it has in the game files. |
| `table.rename_key` | `RenameKey` | `FilesEdited` | Changes a value of a key column of a table in every table of an open pack, including the columns referencing it and the loc keys generated from it. |
| `table.add_key_deletes` | `AddKeyDeletes` | `FilesEdited` | Adds rows to a key deletes table of an open pack, to delete keys of a table in the game. |
| `table.export_tsv` | `ExportTsv` | `Done` | Writes a table to a TSV file. |
| `table.import_tsv` | `ImportTsv` | `TableEdited` | Replaces a table of an open pack with the contents of a TSV file, keeping its GUID. |

## Schema

| Method | Request | Response | Description |
|--------|---------|----------|-------------|
| `schema.tables` | `ListSchemaTables` | `SchemaTables` | Lists the tables of the schema, with the versions it has definitions for. |
| `schema.patch_column` | `PatchColumn` | `Done` | Changes how the schema describes a column, with a local patch. |
| `schema.remove_patches` | `RemovePatches` | `Done` | Removes the local patches of a table, or of one of its columns, and reloads the schema. |
| `schema.update` | `UpdateSchemas` | `SessionStatus` | Downloads the latest schemas, reloads the selected game's one, and rebuilds the dependencies. Runs as a job. |
| `schema.update_from_assembly_kit` | `UpdateSchemaFromAssemblyKit` | `SessionStatus` | Updates the schema of the selected game with the tables of its Assembly Kit, and saves it. Runs as a job. |
| `schema.raw_definitions` | `GetRawDefinitions` | `RawDefinitions` | Returns definitions of a table as the schema stores them, to edit them. |
| `schema.set_definition` | `SetDefinition` | `Done` | Adds a definition to the schema of the selected game, or replaces the one with its version, then saves and reloads the schema. |
| `schema.delete_definition` | `DeleteDefinition` | `Done` | Removes a definition from the schema of the selected game, then saves and reloads the schema. |
| `schema.referencing_columns` | `GetReferencingColumns` | `ReferencingColumns` | Returns the columns of other tables referencing each column of a table. |
| `schema.patches` | `GetTablePatches` | `TablePatches` | Returns the patches applied to the columns of a table. |
| `schema.import_patches` | `ImportPatches` | `Done` | Adds patches to the schema of the selected game itself, and saves it. |
| `schema.missing_definitions` | `GetMissingDefinitions` | `MissingDefinitions` | Returns the tables of an open pack with rows that can't be decoded with the schema. |

## References

| Method | Request | Response | Description |
|--------|---------|----------|-------------|
| `references.definition` | `FindDefinition` | `RowLocation` | Finds the row where a value of a referenced table is defined. |
| `references.usages` | `FindUsages` | `Usages` | Finds the rows of other tables referencing a value of a table. |
| `references.loc` | `FindLoc` | `RowLocation` | Finds the row of a Loc file with a key. |
| `references.loc_source` | `GetLocSource` | `LocSourceLookup` | Returns the table, column and row a loc key belongs to. |
| `references.row_locs` | `GetRowLocs` | `RowLocs` | Returns the loc entries of a row of a DB table: the loc key of each localised column, with its text. |
| `references.table_data` | `GetTableReferenceData` | `TableReferenceData` | Returns the values each column of a table can reference, with their lookups. |
| `references.values` | `GetReferenceValues` | `ReferenceValues` | Returns the values a reference column of a table can have, with their display text. |

## Diagnostics

| Method | Request | Response | Description |
|--------|---------|----------|-------------|
| `diagnostics.run` | `RunDiagnostics` | `DiagnosticsSummary` | Checks the open packs for problems, keeping the results for [`ListDiagnostics`]. Runs as a job. |
| `diagnostics.list` | `ListDiagnostics` | `DiagnosticList` | Returns results of the last check, optionally filtered. |
| `diagnostics.report` | `GetDiagnosticsReport` | `Diagnostics` | Returns the results of the last diagnostics check, as the check returns them. |
| `diagnostics.ignore` | `IgnoreDiagnostics` | `Done` | Makes the next checks of a pack skip some results. |

## Search

| Method | Request | Response | Description |
|--------|---------|----------|-------------|
| `search.report` | `GetSearchReport` | `SearchReport` | Returns the last search with all its matches, as the search returns them. |
| `search.run` | `RunSearch` | `SearchSummary` | Searches text, keeping the matches for [`ListSearchMatches`]. Runs as a job. |
| `search.matches` | `ListSearchMatches` | `SearchMatchList` | Returns matches of the last search, optionally filtered. |
| `search.replace` | `ReplaceSearchMatches` | `SearchReplaced` | Replaces matches of the last search, and searches the edited files again. |

## Notes

| Method | Request | Response | Description |
|--------|---------|----------|-------------|
| `notes.list` | `ListNotes` | `NoteList` | Returns the notes of an open pack for a path. |
| `notes.add` | `AddNote` | `NoteEntry` | Attaches a note to a file or folder of an open pack. |
| `notes.delete` | `DeleteNote` | `Done` | Deletes a note of an open pack. |

## Tools

| Method | Request | Response | Description |
|--------|---------|----------|-------------|
| `tools.optimizer_options` | `GetOptimizerOptions` | `OptimizerOptionValues` | Returns the optimizer options, as the server's settings have them. |
| `tools.optimize` | `OptimizePack` | `FilesChanged` | Removes data of an open pack that's identical to the vanilla one, or unneeded. Runs as a job. |
| `tools.patch_siege_ai` | `PatchSiegeAi` | `SiegeAiPatched` | Patches the siege maps of an open pack, so the AI can use them. |
| `tools.pack_map` | `PackMap` | `FilesChanged` | Adds the tiles and tile maps of a map exported by Terry to an open pack. |
| `tools.generate_missing_locs` | `GenerateMissingLocs` | `FilesEdited` | Adds empty loc entries for the localised columns of the tables of the open packs that have none. |
| `tools.update_anim_ids` | `UpdateAnimIds` | `FilesEdited` | Offsets the animation ids of an open pack, like after a game update moves them. |
| `tools.anims_by_skeleton` | `AnimsBySkeleton` | `FilePaths` | Returns the paths of the animations using a skeleton, in the open packs and the dependencies. |
| `tools.export_gltf` | `ExportGltf` | `Done` | Exports a RigidModel to a glTF file, with its textures. |
| `tools.set_video_format` | `SetVideoFormat` | `Done` | Changes the format of a ca_vp8 video of an open pack. |
| `tools.live_export` | `LiveExport` | `Done` | Exports the scripts and UI files of an open pack to the game's data folder, to test them without saving the pack. |
| `tools.init_mymod` | `InitMyMod` | `MyModCreated` | Creates the folder of a new MyMod, with editor configs for Lua scripting. |
| `lua.run_tests` | `RunLuaTests` | `LuaTestResults` | Runs Lua tests against the scripts of the game and the open packs, outside of the game. Runs as a job. |
| `startpos.campaigns` | `GetStartposCampaigns` | `StartposCampaigns` | Returns the campaigns a startpos can be built for. |
| `startpos.start` | `StartStartpos` | `Done` | Prepares the Assembly Kit to build a startpos with the tables of an open pack, and launches the game to build it. |
| `startpos.finish` | `FinishStartpos` | `FilesEdited` | Imports the startpos built by the game into the pack, or cancels the build, and cleans up the build files. |
| `tools.plugin_scripts` | `ListPluginScripts` | `PluginScripts` | Returns the plugin scripts in the scripts folder of the config. |
| `tools.run_plugin_script` | `RunPluginScript` | `PluginScriptRun` | Runs a plugin script over files of an open pack, reading back the files it changes. |
| `lua.hovers` | `GetLuaHovers` | `LuaHovers` | Returns the docs of the game's Lua API for the symbols of a Lua script, to show on hover. |
| `ceo.traits` | `ListTraitCeos` | `TraitCeos` | Returns the trait CEOs of the Assembly Kit tables. |
| `ceo.add_entries` | `AddCeoEntries` | `FilesEdited` | Adds CEO entries (armour, career, traits and their locs) to an open pack. |
| `ceo.build` | `BuildCeo` | `Done` | Builds `ceo_data.ccd` in the Assembly Kit from the CEO tables of an open pack, running BOB. |
| `ceo.import` | `ImportCeo` | `FilesEdited` | Imports the `ceo_data.ccd` built by `ceo.build` into an open pack. |

## Translations

| Method | Request | Response | Description |
|--------|---------|----------|-------------|
| `translations.pack` | `GetPackTranslation` | `PackTranslation` | Returns the whole translation of an open pack to a language, as the translator works with it. |
| `translations.submit` | `SubmitTranslation` | `TranslationSubmission` | Submits the saved translation of a pack to the Translation Hub, opening or updating a pull request. |
| `translations.list` | `ListTranslations` | `Translations` | Returns the translation of the texts of an open pack to a language. |
| `translations.generate_vanilla` | `GenerateVanillaTexts` | `VanillaTextsAvailable` | Generates the vanilla texts of a language from the game's locale packs, so translations to and from it can reuse them. |

## GitHub

| Method | Request | Response | Description |
|--------|---------|----------|-------------|
| `github.sign_in_start` | `StartGitHubSignIn` | `DeviceCode` | Starts a GitHub sign-in, returning the code the user has to enter on GitHub. |
| `github.sign_in_poll` | `PollGitHubSignIn` | `GitHubSignInState` | Checks if the user approved a sign-in. |
| `github.account` | `GetGitHubAccount` | `GitHubAccount` | Returns the GitHub account the user is signed in as. |
| `github.sign_out` | `SignOutOfGitHub` | `Done` | Signs out of GitHub, deleting the stored sign-in. |

## Updates

| Method | Request | Response | Description |
|--------|---------|----------|-------------|
| `updates.check` | `CheckUpdate` | `UpdateStatus` | Checks if there is an update. |
| `updates.apply` | `ApplyUpdate` | `Done` | Downloads an update. |

## Jobs

| Method | Request | Response | Description |
|--------|---------|----------|-------------|
| `job.status` | `GetJobStatus` | `JobStatus` | Returns the state of a job. |
| `job.wait` | `WaitForJob` | `JobStatus` | Waits for a job to end, and returns its state. |
| `job.cancel` | `CancelJob` | `Done` | Cancels a job that hasn't started yet. |

