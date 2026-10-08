<!-- Generated from the API types by `rpfm_ipc/src/api/method_docs.rs`. Don't edit it by hand. -->

# Methods

Every method of the API, grouped by domain. Each domain's page lists the params and the result of its methods, generated from the request and response structs of `rpfm_ipc::api`. Methods marked as jobs answer right away with a job ID, and send their result when the job ends (see [Jobs](./ws-protocol.md#jobs)).

Methods meant for clients with their own views of the files, like the UI, return RPFM's own types (decoded files, typed search matches, full diagnostics), shown as `any` in their pages. Simpler clients should prefer the paginated methods next to them, like `table.rows`, `search.matches` or `diagnostics.list`.

## [Session and dependencies](./methods/session.md)

| Method | Description |
|--------|-------------|
| [`session.status`](./methods/session.md#sessionstatus) | Returns the selected game, what's loaded for it, and the open packs. |
| [`session.configure`](./methods/session.md#sessionconfigure) | Replaces the settings the session runs with. |
| [`session.disconnect`](./methods/session.md#sessiondisconnect) | Tells the server the client is closing, so its WebSocket session is removed right away instead of waiting for the timeout. Only for WebSocket clients. |
| [`session.set_game`](./methods/session.md#sessionset_game) | Selects a game, loading its schema and, optionally, its dependencies. Runs as a job. |
| [`dependencies.generate_cache`](./methods/session.md#dependenciesgenerate_cache) | Generates the dependencies cache of the selected game from its files and Assembly Kit, and loads it. Runs as a job. |
| [`dependencies.rebuild`](./methods/session.md#dependenciesrebuild) | Reloads the dependencies of the selected game, like after changing the packs the open packs depend on. Runs as a job. |
| [`dependencies.tables`](./methods/session.md#dependenciestables) | Returns the tables of the game files, and the startpos, twad and CEO tables of the schema, with their version. |
| [`dependencies.info`](./methods/session.md#dependenciesinfo) | Returns the files of the dependencies of the selected game. |
| [`dependencies.table_data`](./methods/session.md#dependenciestable_data) | Returns every decoded table of a type in the game files and the parent packs. |

## [Packs](./methods/packs.md)

| Method | Description |
|--------|-------------|
| [`pack.info`](./methods/packs.md#packinfo) | Returns the details of an open pack. |
| [`pack.new`](./methods/packs.md#packnew) | Creates a new empty pack. It has no path on disk until it's saved with a path. |
| [`pack.open`](./methods/packs.md#packopen) | Opens one or more packs from disk, merged into a single one. |
| [`pack.open_vanilla`](./methods/packs.md#packopen_vanilla) | Opens all the vanilla packs of the selected game, merged into a single one. |
| [`pack.close`](./methods/packs.md#packclose) | Closes an open pack, discarding unsaved changes. |
| [`pack.close_all`](./methods/packs.md#packclose_all) | Closes all open packs, discarding unsaved changes. |
| [`pack.save`](./methods/packs.md#packsave) | Saves an open pack to disk. |
| [`pack.update`](./methods/packs.md#packupdate) | Changes properties of an open pack. Only the fields set are changed. |
| [`pack.settings`](./methods/packs.md#packsettings) | Returns the settings of an open pack. |
| [`pack.update_settings`](./methods/packs.md#packupdate_settings) | Changes settings of an open pack. Only the keys set are changed. |

## [Files](./methods/files.md)

| Method | Description |
|--------|-------------|
| [`files.list`](./methods/files.md#fileslist) | Lists the files of a source, sorted by path. |
| [`files.create`](./methods/files.md#filescreate) | Creates a new empty file in an open pack. |
| [`files.add_from_disk`](./methods/files.md#filesadd_from_disk) | Adds files and folders from disk to an open pack. |
| [`files.copy`](./methods/files.md#filescopy) | Copies files and folders from any source into an open pack, keeping their paths. |
| [`files.delete`](./methods/files.md#filesdelete) | Deletes files and folders from an open pack. |
| [`files.rename`](./methods/files.md#filesrename) | Renames or moves files and folders of an open pack. |
| [`files.duplicate`](./methods/files.md#filesduplicate) | Copies files of an open pack in the same pack, adding a number to their names. |
| [`files.extract`](./methods/files.md#filesextract) | Extracts files and folders of an open pack, the game files or the parent packs to disk. |
| [`file.read`](./methods/files.md#fileread) | Returns the contents of a file of any source. |
| [`file.write`](./methods/files.md#filewrite) | Replaces the contents of a file of an open pack. |
| [`animpack.list`](./methods/files.md#animpacklist) | Lists the files inside an AnimPack of any source. |
| [`animpack.add`](./methods/files.md#animpackadd) | Copies files of an open pack into an AnimPack of an open pack. |
| [`animpack.extract`](./methods/files.md#animpackextract) | Copies files of an AnimPack of any source into an open pack. |
| [`animpack.delete`](./methods/files.md#animpackdelete) | Deletes files from an AnimPack of an open pack. |
| [`file.view_data`](./methods/files.md#fileview_data) | Returns a file decoded as a client opening it in a view needs it. |
| [`files.info`](./methods/files.md#filesinfo) | Returns the info of files of an open pack, like their original pack and last modification time. |
| [`files.paste`](./methods/files.md#filespaste) | Copies or moves files and folders of open packs into a folder of an open pack. |
| [`files.from_all_sources`](./methods/files.md#filesfrom_all_sources) | Returns files and folders found in the open packs, the parent packs and the game files. |
| [`files.save_files`](./methods/files.md#filessave_files) | Adds files to an open pack, replacing the ones with the same path, optionally optimizing the pack after it. |
| [`file.open_external`](./methods/files.md#fileopen_external) | Extracts a file of an open pack to a temporary folder, and opens it in the system's default program for its type. |
| [`file.save_external`](./methods/files.md#filesave_external) | Replaces a file of an open pack with the file on disk an external program edited. |

## [Tables](./methods/tables.md)

| Method | Description |
|--------|-------------|
| [`table.info`](./methods/tables.md#tableinfo) | Returns the definition and row count of a table. |
| [`table.rows`](./methods/tables.md#tablerows) | Returns rows of a table, optionally filtered and with only some columns. |
| [`schema.definition`](./methods/tables.md#schemadefinition) | Returns the columns of a table as defined in the schema. |
| [`table.edit`](./methods/tables.md#tableedit) | Edits rows of a DB or Loc table in an open pack. |
| [`table.column_values`](./methods/tables.md#tablecolumn_values) | Returns the distinct values of a column of a table, sorted. |
| [`table.unused_numbers`](./methods/tables.md#tableunused_numbers) | Returns numbers no row uses yet in an integer column of a table, for new rows needing unique ones, like ids. |
| [`table.merge`](./methods/tables.md#tablemerge) | Merges tables of the same type of an open pack into a new one. |
| [`table.upgrade`](./methods/tables.md#tableupgrade) | Updates a table of an open pack to the version it has in the game files. |
| [`table.rename_key`](./methods/tables.md#tablerename_key) | Changes a value of a key column of a table in every table of an open pack, including the columns referencing it and the loc keys generated from it. |
| [`table.add_key_deletes`](./methods/tables.md#tableadd_key_deletes) | Adds rows to a key deletes table of an open pack, to delete keys of a table in the game. |
| [`table.export_tsv`](./methods/tables.md#tableexport_tsv) | Writes a table to a TSV file. |
| [`table.import_tsv`](./methods/tables.md#tableimport_tsv) | Replaces a table of an open pack with the contents of a TSV file, keeping its GUID. |

## [Schema](./methods/schema.md)

| Method | Description |
|--------|-------------|
| [`schema.tables`](./methods/schema.md#schematables) | Lists the tables of the schema, with the versions it has definitions for. |
| [`schema.patch_column`](./methods/schema.md#schemapatch_column) | Changes how the schema describes a column, with a local patch. |
| [`schema.remove_patches`](./methods/schema.md#schemaremove_patches) | Removes the local patches of a table, or of one of its columns, and reloads the schema. |
| [`schema.update`](./methods/schema.md#schemaupdate) | Downloads the latest schemas, reloads the selected game's one, and rebuilds the dependencies. Runs as a job. |
| [`schema.update_from_assembly_kit`](./methods/schema.md#schemaupdate_from_assembly_kit) | Updates the schema of the selected game with the tables of its Assembly Kit, and saves it. Runs as a job. |
| [`schema.raw_definitions`](./methods/schema.md#schemaraw_definitions) | Returns definitions of a table as the schema stores them, to edit them. |
| [`schema.set_definition`](./methods/schema.md#schemaset_definition) | Adds a definition to the schema of the selected game, or replaces the one with its version, then saves and reloads the schema. |
| [`schema.delete_definition`](./methods/schema.md#schemadelete_definition) | Removes a definition from the schema of the selected game, then saves and reloads the schema. |
| [`schema.referencing_columns`](./methods/schema.md#schemareferencing_columns) | Returns the columns of other tables referencing each column of a table. |
| [`schema.patches`](./methods/schema.md#schemapatches) | Returns the patches applied to the columns of a table. |
| [`schema.import_patches`](./methods/schema.md#schemaimport_patches) | Adds patches to the schema of the selected game itself, and saves it. |
| [`schema.missing_definitions`](./methods/schema.md#schemamissing_definitions) | Returns the tables of an open pack with rows that can't be decoded with the schema. |

## [References](./methods/references.md)

| Method | Description |
|--------|-------------|
| [`references.definition`](./methods/references.md#referencesdefinition) | Finds the row where a value of a referenced table is defined. |
| [`references.usages`](./methods/references.md#referencesusages) | Finds the rows of other tables referencing a value of a table. |
| [`references.loc`](./methods/references.md#referencesloc) | Finds the row of a Loc file with a key. |
| [`references.loc_source`](./methods/references.md#referencesloc_source) | Returns the table, column and row a loc key belongs to. |
| [`references.row_locs`](./methods/references.md#referencesrow_locs) | Returns the loc entries of a row of a DB table: the loc key of each of its localised columns, with its text. |
| [`references.table_data`](./methods/references.md#referencestable_data) | Returns the values each column of a table can reference, with their lookups. |
| [`references.values`](./methods/references.md#referencesvalues) | Returns the values a reference column of a table can have, with their display text. |

## [Diagnostics](./methods/diagnostics.md)

| Method | Description |
|--------|-------------|
| [`diagnostics.run`](./methods/diagnostics.md#diagnosticsrun) | Checks the open packs for problems, keeping the results for `diagnostics.list`. Runs as a job. |
| [`diagnostics.list`](./methods/diagnostics.md#diagnosticslist) | Returns results of the last check, optionally filtered. |
| [`diagnostics.report`](./methods/diagnostics.md#diagnosticsreport) | Returns the results of the last diagnostics check, as the check returns them. |
| [`diagnostics.ignore`](./methods/diagnostics.md#diagnosticsignore) | Makes the next checks of a pack skip some results. Saved in the pack's settings. |

## [Search](./methods/search.md)

| Method | Description |
|--------|-------------|
| [`search.report`](./methods/search.md#searchreport) | Returns the last search with all its matches, as the search returns them. |
| [`search.run`](./methods/search.md#searchrun) | Searches text, keeping the matches for `search.matches`. Runs as a job. |
| [`search.matches`](./methods/search.md#searchmatches) | Returns matches of the last search, optionally filtered. |
| [`search.replace`](./methods/search.md#searchreplace) | Replaces matches of the last search, and searches the edited files again. |

## [Notes](./methods/notes.md)

| Method | Description |
|--------|-------------|
| [`notes.list`](./methods/notes.md#noteslist) | Returns the notes of an open pack for a path. |
| [`notes.add`](./methods/notes.md#notesadd) | Attaches a note to a file or folder of an open pack. |
| [`notes.delete`](./methods/notes.md#notesdelete) | Deletes a note of an open pack. |

## [Tools](./methods/tools.md)

| Method | Description |
|--------|-------------|
| [`tools.optimizer_options`](./methods/tools.md#toolsoptimizer_options) | Returns the optimizer options, as the server's settings have them. |
| [`tools.optimize`](./methods/tools.md#toolsoptimize) | Removes data of an open pack that's identical to the vanilla one, or unneeded. Runs as a job. |
| [`tools.patch_siege_ai`](./methods/tools.md#toolspatch_siege_ai) | Patches the siege maps of an open pack, so the AI can use them. Warhammer games only. |
| [`tools.pack_map`](./methods/tools.md#toolspack_map) | Adds the tiles and tile maps of a map exported by Terry to an open pack. |
| [`tools.generate_missing_locs`](./methods/tools.md#toolsgenerate_missing_locs) | Adds empty loc entries for the localised columns of the tables of the open packs that have none. |
| [`tools.update_anim_ids`](./methods/tools.md#toolsupdate_anim_ids) | Offsets the animation ids of an open pack, like after a game update moves them. |
| [`tools.anims_by_skeleton`](./methods/tools.md#toolsanims_by_skeleton) | Returns the paths of the animations using a skeleton, in the open packs and the dependencies. |
| [`tools.export_gltf`](./methods/tools.md#toolsexport_gltf) | Exports a RigidModel to a glTF file, with its textures. |
| [`tools.set_video_format`](./methods/tools.md#toolsset_video_format) | Changes the format of a ca_vp8 video of an open pack. |
| [`tools.live_export`](./methods/tools.md#toolslive_export) | Exports the scripts and UI files of an open pack to the game's data folder, to test them without saving the pack. |
| [`tools.init_mymod`](./methods/tools.md#toolsinit_mymod) | Creates the folder of a new MyMod, with editor configs for Lua scripting. |
| [`lua.run_tests`](./methods/tools.md#luarun_tests) | Runs Lua tests against the scripts of the game and the open packs, outside of the game. Runs as a job. |
| [`startpos.campaigns`](./methods/tools.md#startposcampaigns) | Returns the campaigns a startpos can be built for. |
| [`startpos.start`](./methods/tools.md#startposstart) | Prepares the Assembly Kit to build a startpos with the tables of an open pack, and launches the game to build it. |
| [`startpos.finish`](./methods/tools.md#startposfinish) | Imports the startpos built by the game into the pack, or cancels the build, and cleans up the build files. |
| [`tools.plugin_scripts`](./methods/tools.md#toolsplugin_scripts) | Returns the plugin scripts in the scripts folder of the config. |
| [`tools.run_plugin_script`](./methods/tools.md#toolsrun_plugin_script) | Runs a plugin script over files of an open pack, reading back the files it changes. |
| [`lua.hovers`](./methods/tools.md#luahovers) | Returns the docs of the game's Lua API for the symbols of a Lua script, to show on hover. |
| [`ceo.traits`](./methods/tools.md#ceotraits) | Returns the trait CEOs of the Assembly Kit tables. |
| [`ceo.add_entries`](./methods/tools.md#ceoadd_entries) | Adds CEO entries (armour, career, traits and their locs) to an open pack. |
| [`ceo.build`](./methods/tools.md#ceobuild) | Builds `ceo_data.ccd` in the Assembly Kit from the CEO tables of an open pack, running BOB. |
| [`ceo.import`](./methods/tools.md#ceoimport) | Imports the `ceo_data.ccd` built by `ceo.build` into an open pack. |

## [Translations](./methods/translations.md)

| Method | Description |
|--------|-------------|
| [`translations.pack`](./methods/translations.md#translationspack) | Returns the whole translation of an open pack to a language, as the translator works with it. |
| [`translations.submit`](./methods/translations.md#translationssubmit) | Submits the saved translation of a pack to the Translation Hub, opening or updating a pull request. |
| [`translations.list`](./methods/translations.md#translationslist) | Returns the translation of the texts of an open pack to a language. |
| [`translations.generate_vanilla`](./methods/translations.md#translationsgenerate_vanilla) | Generates the vanilla texts of a language from the game's locale packs, so translations to and from it can reuse them. |

## [GitHub](./methods/github.md)

| Method | Description |
|--------|-------------|
| [`github.sign_in_start`](./methods/github.md#githubsign_in_start) | Starts a GitHub sign-in, returning the code the user has to enter on GitHub. |
| [`github.sign_in_poll`](./methods/github.md#githubsign_in_poll) | Checks if the user approved a sign-in. Once approved, the server keeps the sign-in. |
| [`github.account`](./methods/github.md#githubaccount) | Returns the GitHub account the user is signed in as. |
| [`github.sign_out`](./methods/github.md#githubsign_out) | Signs out of GitHub, deleting the stored sign-in. |

## [Updates](./methods/updates.md)

| Method | Description |
|--------|-------------|
| [`updates.check`](./methods/updates.md#updatescheck) | Checks if there is an update. |
| [`updates.apply`](./methods/updates.md#updatesapply) | Downloads an update. Schemas can't be updated with this, as they need reloading: use `schema.update`. |

## [Jobs](./methods/jobs.md)

| Method | Description |
|--------|-------------|
| [`job.status`](./methods/jobs.md#jobstatus) | Returns the state of a job. |
| [`job.wait`](./methods/jobs.md#jobwait) | Waits for a job to end, and returns its state. |
| [`job.cancel`](./methods/jobs.md#jobcancel) | Cancels a job that hasn't started yet. Running jobs can't be cancelled. |
