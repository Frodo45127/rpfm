# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project doesn't adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

If you're looking for the changes included in the latest beta (against the latest stable version), check the unreleased section.

## [2.5.3]
### Added
- Implemented `Diagnostic Check` command on the CLI.
- Implemented `Incorrect Game Path` diagnostic.

### Changed
- Updated dependencies.
- Revised diagnostic ignore code to allow ígnoring only specific diagnostics on specific cells.
- `Create DB` button is now always enabled, but it'll fail with an error if you do not have the schemas downloaded or the dependencies cache generated.

### Fixed
- Fixed cells being marked as modified on cloning.
- Fixed infinite schema download bug.
- Fixed random lockups shortly after a table edition.
- Fixed ignoring a diagnostic on a key column causing other diagnostics using that key column to generate false positives.
- Fixed `Generate Dependencies Cache` incorrectly succeeding when the Game Path was incorrect.
- Fixed typos on install/uninstall messages.

## [2.5.2]
### Added
- Table colours can now be changed on the settings.

### Changed
- Table/File status markers are now cleared on save.
- Tweaked table status markers and colours.
- Restructured settings window.

### Fixed
- Fixed paths diagnostic giving a lot of false positives.
- Fixed paths diagnostic not checking paths correctly if they end with "/".
- Fixed paths diagnostic not checking folder paths correctly if the folder exists but has no files.
- Fixed paths diagnostic not checking paths correctly if casing doesn't match.
- Fixed paths diagnostic failing to find folders on parent mods.
- Fixed paths diagnostic failing to find folders on game packfiles.
- Fixed data not being sorted correctly when loading it to a combo of a table.
- Fixed CTD on diagnostic check.
- Fixed empire's `Voices.pack` failing to open.
- Fixed `Load all CA PackFiles` not working on Empire.
- Fixed `Generate Dependencies Cache` asking for regeneration on start on Empire/Napoleon.
- Fixed `Load all CA PackFiles` not only loading CA PackFiles, but also every single pack on Empire/Napoleon.
- Fixed `Generate Dependencies Cache` using data from the previous game selected.
- Fixed CTD when double-clicking certain diagnostics.
- Fixed `Insert Rows` not marking new rows as added if there was not a cell selected before the insertion.
- Fixed tables not using the correct marker colours for their theme.
- Fixed previously opened Packs not opening again with `Add from PackFile`.

## [2.5.1]
### Changed
- Removed requeriment of Assembly Kit for Generation of Dependencies Cache.
- Implemented Generation of Dependencies Cache for Empire and Napoleon.

### Fixed
- Fixed extremely poor performance of the diagnostics tool.
- Fixed issue where paths starting with `/` were ignored in the ignored paths for Diagnostic checks.

## [2.5.0]
### Added
- Implemented `Invalid Loc Key` diagnostic.
- Implemented `Invalid PackFile Name` diagnostic.
- Implemented `Table name ends in number` diagnostic.
- Implemented `Table name contains spaces` diagnostic.
- Implemented `Table is datacoring` diagnostic.
- Implemented `Dependencies Cache not yet generated` diagnostic.
- Implemented `Dependencies Cache outdated` diagnostic.
- Implemented `Dependencies Cache could not be loaded` diagnostic.
- Implemented `Path/File in field not found` diagnostic.
- Implemented `Debug` PackedFile View.
- Implemented support for UnitVariant (Shogun 2/Napoleon/Empire).
- Implemented support for RigidModels (new editor by [@phazer](https://github.com/mr-phazer)).
- Implemented `Game-Wide` caché (replaces the old PAKs).
- Implemented support for steam's `MFH` packs.
- Implemented read support for modern DDS files (by [@phazer](https://github.com/mr-phazer)).
- Implemented support to open references from out of the Packfile when using `Go To Definition` or `Go To Loc`.
- Implemented support for alternative version 0 definitions.
- Reworked cell painting on tables, so it should no longer left unreadable cells on painting.
- Implemented support for ignoring specific diagnostics per PackFile.
- Key columns now have a distinct background.
- Added instructions to the AnimPack view.
- Added a dialog before generating the dependencies cache.
- Added a dialog before optimizing a PackFile, explaining what the optimizer does, and asking the user to make a backup before using it.
- Added clear filter buttons to `Add From PackFile` and `AnimPack` filters.

### Changed
- Improved memory usage when extracting large amounts of files in one go.
- Updated Qt dependencies to 5.15.2 (this time for real).
- Removed greying out on Autosave.
- Re-enable automatic crash report with Sentry.
- Now double-clicking diagnostics related to RPFM configuration opens the relevant configuration.
- Open From Data should no longer check subdirs for Packs.
- `Recent Files` list now should work across instances.
- Updated manual.

### Fixed
- Fixed false positives on the diagnostics tool after adding new tables to a PackFile.
- Fixed `Update Table` not using the default value for the new columns.
- Fixed smart delete deleting the wrong cell if the columns were sorted.
- Fixed the infamous `Ambiguous Shortcut Ctrl+S` bug when trying to save with a Text PackedFile open.
- Fixed incorrect original PackFile being reported on the PackedFile's tooltips when using Load All CA PAckFiles or opening multiple PackFiles at once.
- Fixed scroll/selection not working on TreeView when opening diagnostics/tables.
- Fixed duplicate diagnostics not checking across files.
- Fixed duplicate keys diagnostics not working on tables with just one key column.
- Fixed diagnostics not being painted to tables after opening them.
- Fixed RPFM failing to reload the dependencies on cache generation.
- Fixed jpg images not loading.
- Fixed some paste operations not pasting where they should if a filter/sorting was applied to the table before the operation.
- Fixed table not updating correctly after a certain operations.
- Fixed warnings.
- Fixed a hang on opening/creating PackFiles if the user had a game installed with missing Packs.
- Fixed a few issues related to the dependencies cache.
- Fixed CTD on certain table view reloads.
- Fixed performance issues when toggling all diagnostics filters at once.
- Fixed certain diagnostics being duplicated on checking open PackedFiles.
- Fixed certain diagnostics not painting all the cells they should.
- Fixed CTD on trying to reopen an already open PackFile with `Add from PackedFile`.
- Fixed some issues with the CTD reporting logic not always actually reporting.
- Fixed some issues with the CTD reporting logic with caused backend crashes to cause the "Grey Screen of Death".
- Fixed a semi-random CTD that happened when RPFM could not access for a moment to a file on disk.
- Fixed a CTD that happened when a update download ended up with an incomplete file.
- Fixed `Paste as New Row` not properly marking cells as added.
- Fixed CTD that happenend sometimes when trying to use the table decoder.
- Fixed false positive on diagnostics when a cell was of a numeric type, it referenced another cell, and had 0 as value.

### Known Issues
- The `Duplicated Combined Key` is not very efficient on mods with tables with large amounts of entries (+5k rows). If checking your PackFile takes too long, you can disable that diagnostic for that PackFile in its PackFile Settings.
- The new RigidModel editor is still in beta, and there are some RigidModels it cannot read properly/cause crashes at reading them. If you experience instabilities while using it, you can disable it in the Settings.

## [2.4.3]
### Added
- Implemented a more robust corruption detection system.
- Implemented `Rescue PackFile` feature, to rescue uncorrupted files from PackFiles that cannot be saved due to corruption.
- Implemented `OR` filters for tables, through groups.

### Changed
- You can now choose to show/hide blank cells on table filters.
- Removed filter delay in everything but LineEdits, so it's only delayed when writing.
- The window now it's darkened when adding files, to show it's doing something.
- Optimized PackFile loading logic by about 30-40% (takes less time to open PackFiles).
- Optimized TreeView building logic by about 70% (takes wwaaaaay less time to build the TreeView after opening a PackFile).
- Reverted changes to the Extract dialog on 2.4, as not everyone (not even me) was too happy with them.
- Reworked internal PackFile type detection logic.
- AnimTables are no longer required to have the name `animation_tables.bin` to be opened. Now RPFM detects them correctly, as long as they're in in `animation/animation_tables/` and their name ends in `_tables.bin`.
- MatchedCombat files are no longer required to have the name `attila_generated.bin` to be opened. Now RPFM detects them correctly, as long as they're in in `animation/matched_combat/` and their name ends in `.bin`.
- `Install` feature will now try to install the PackFile image too if it finds it.
- Changed default `Install/Uninstall` shortcuts.

### Fixed
- Fixed incorrect optimization in the definition guesser.
- Fixed rare hang on adding files to a PackFile.
- Fixed empty tab name when opening files with external tools.
- Fixed `Load All CA PackFiles` not working with older games without manifests.
- Fixed 2 instances were RPFM left a thread running on close, leaving a process doing nothing but consuming memory in the background.
- Fixed a bug that caused clicking the button `-` on filters to remove the bottom filter, not the one you clicked.
- Fixed a rare CTD when the Autosave kicked in while a heavy load operation was taking place.
- Fixed double "Are you sure?" dialog on close from the `Quit` action.
- Fixed RPFM not remembering its own layout.

## [2.4.2]
### Fixed
- Fixed hang when updating tables.

## [2.4.1]
### Fixed
- Fixed CTD on editing integer cells.
- Fixed lost focus while editing string cells.

## [2.4.0]
### Added
- Implemented `To Json` command on the CLI, to convert the schemas to Json.
- Implemented context menu for PackedFile View Tabs.
- Implemented `Close Other Tabs` feature.
- Implemented `Close Other Tabs to the Left` feature.
- Implemented `Close Other Tabs to the Right` feature.
- Implemented `Disable autosaves` packfile setting.
- Implemented `Autosave Amount` setting.
- Implemented `Clear autosave folder` button on settings.
- Implemented `Clear schema folder` button on settings.
- Implemented `New AnimPack` feature.
- Implemented `Restart` button on update dialog.
- New  `rpfm.exe` executable to launch RPFM UI with self-restarting capabilities.
- Added changelog link to the "RPFM updated successfully" dialog.
- Implemented `Import` command to quickly import everything from a MyMod's Assets folder into a MyMod ([@chadvandy](https://github.com/chadvandy)).
- Implemented `Export` command to quickly export everything from a MyMod into its Asset Folder ([@chadvandy](https://github.com/chadvandy)).
- Implemented `Files to Ignore on Import` PackFile Setting to blacklist files from autoimporting when using the new `Import` MyMod command ([@chadvandy](https://github.com/chadvandy)).
- Implemented diagnostic description tooltip when hovering the mouse over them, to know what each diagnostic means, and how to fix it.
- Implemented an `Apply Settings` button on the PackFile Settings view, to instantly apply those settings without having to save the PackFile.
- Implemented `Rename References` feature.
- Implemented `Clear` button for TreeView and Table filters.
- Implemented `Delete Filtered-out Rows` feature.
- Implemented `Generate Ids` feature.
- Implemented `Check PackFile` and `Check Open PackedFiles` buttons to manually trigger diagnostics checks.
- Implemented `Go To Definition` feature.
- Implemented `Go To Loc` feature.

### Changed
- Small performance optimizations for querying for dependency data.
- Updated dependencies.
- Improved definition importer performance, both in time and memory consumption.
- Improved dependency resolving time by 40-60%.
- Improved performance when swapping/closing PackedFiles (it means it doesn't take a second to close a table).
- Improved performance of the following table operations:
    + Paste
    + Paste as new row
    + Delete
    + Delete rows
    + Rewrite selection
- Improved Schema/Template updater to be more reliable.
- Reworked AnimPack View.
- UpdateXXXX folders are now deleted after an update.
- Added icon to the cli tool.
- Improved diagnostics' blacklisting logic to allow blacklisting of entire folders, and of particular columns.
- Reworked `Install/Uninstall` commands to work with any PackFile, not only MyMods, and moved both commands to the `PackFile` menu ([@chadvandy](https://github.com/chadvandy)).
- Empty rows are now shown by default when filtering a table.
- Now each version has a name (why not?).
- Improved responsiness during diagnostics checks.
- Tweaked timing of diagnostics checks.
- Improved performance when opening PackedFiles.
- `Extract Table` now uses the PackFile's folder as default, then /data, then Rpfm's folder.
- Added small delay before filtering Tables/TreeView to improve performance while filtering.

### Fixed
- Fixed checkbox columns not being sortable.
- Fixed float numbers being copied wrongly.
- Fixed weird colours after a diagnostics check.
- Fixed RPFM failing to parse correctly certain specific sequences of the Assembly Kit.
- Fixed rewrite selection not working properly on integers.
- Fixed missing compile dependency in PKGBUILD for Arch.
- Fixed instance of `Undecoded PackedFile` error.
- Fixed `Missing table definition` debug option not triggering when it should.
- Fixed a bug that caused dependencies of parent mods to not load properly, causing valid data to show as errors in the diagnostics tool.
- Fixed a bug that could cause RPFM to become trap in an infinite PackFile loading loop.
- Fixed a bug that caused the column indexes used in `Rewrite selection` to be incorrect.
- Fixed a bug that caused the `Delete` funtion on tables to not delete properly a row if it had hidden columns.
- Fixed a bug that caused local schema changes to be lost in a schema update.
- Fixed a rare CTD/hang when performing a diagnostics check.
- Fixed `Access is Denied` issue when clearing the schemas folder.
- Fixed an issue that caused RPFM to fail to clean up the schemas folder when trying to update the schemas.
- Fixed a bug that caused the TabBar Context Menu to popup when it shouldn't.
- Fixed a bug that caused `rpfm_macros` to fail to compile in certain systems.
- Fixed a crash when using `Import from Assembly Kit` button in the decoder with Warhammer 2.
- Fixed a bug that caused RPFM to take a few more seconds than it should to start.
- Fixed a bug that caused table filters to not work on checkbox columns.
- Fixed a bug that caused Smart Delete to delete the wrong rows when using a filter.
- Fixed broken links in changelog.
- Fixed MyMod's Import not working when the blacklist for it was empty.
- Fixed MyMod mode not being disabled when opening another PackFile.
- Fixed dependencies not being initialized on new PackFile.
- Fixed tables not being properly updated after certain editions.
- Fixed svg icons not showing up on windows.
- Fixed rare crash/hangs while doing a global search.
- Fixed `Update Table` command closing PackedFiles it shouldn't close.
- Fixed compilation instructions link ([@LenardHess](https://github.com/LenardHess)).

## [2.3.4] - 2020-11-22
### Added
- Implemented improved template controls (experimental, do not use them!!!!).
- Implemented editing support for Texture Arrays.

### Changed
- Increased size of `New PackedFile/Folder` dialogs so the title is not cut.

### Fixed
- Fixed a CTD when trying to perform a local search.
- Fixed a hang when trying to perform a global replace.
- Fixed the local search panel being broken.
- Fixed global search/diagnostics updates not triggering on file deletion.

## [2.3.3] - 2020-11-14
### Added
- Implemented debug setting for changing the Authoring tool in PFH6 PackFiles to CA's.

### Changed
- Increased size of `New PackedFile/Folder` dialogs so the title is not cut.

### Fixed
- Fixed a CTD when trying to open a Dependency PackFile diagnostic.
- Fixed table views not scrolling to matches when trying to open a match from the Global Search/Diagnostics table.
- Fixed some overly-aggressive global search/diagnostics checks.
- Fixed `New PackedFile` dialog having overlayed items.

## [2.3.2] - 2020-11-06
### Changed
- Rewritten the "Open match" logic of Global Search and Diagnostics to not require the item to open being visible in the PackFile TreeView.
- Changed warning cells's color, so it's no longer hard to see in added/modified cells.

### Fixed
- Fixed a CTD when the a table view gets reloaded (for example, doing a global replace).

## [2.3.1] - 2020-11-04
### Fixed
- Fixed a CTD when opening PackFiles if the settings file wasn't initialized beforehand.
- Fixed hide/show checkboxes hiding/showing the wrong columns on tables.
- Fixed light theme error text to be more visible.
- Fixed RPFM not setting PFH6 PackFile's Game Version if the user didn't changed the game selected.

## [2.3.0] - 2020-11-03
### Added
- Implemented support for PFH6 PackFiles (Troy AK-generated mods).
- Implemented more granular filtering for messages of the Diagnostics tool.
- Implemented a checkbox to hide/show all columns of a table at once.
- Implemented a setting to disable item autoexpanding when adding new items to the TreeView.
- Implemented diagnostics support for the Dependency Manager.
- Implemented per-PackFile settings.
- Implemented a PackFile setting to ignore files in the diagnostics checks.
- Implemented multi-column filters for tables.
- Implemented Recent PackFiles list.
- Added a warning to the Dependency Manager.
- Reimplemented the old table cell colours, now based on the diagnostics results.
- Implemented new `RPFM Mod Checker` (Drop its exe into a game´s folder, execute it, and it'll tell you what files are making the game show up as "modded", and for Troy it'll also tell you if you are subscribed to movie files).

### Changed
- Reduced autosave amount to 10.
- Combined Keys diagnostic now show the combined keys.
- Improved release deploying process.
- Improved CA_VP8 UI, including an explanation on how to use it.

### Fixed
- Fixed two CTD that could trigger at many places at random and left no error log.
- Fixed a CTD when trying to open an "Outdated Table" diagnostics result.
- Fixed a weird memory leak in the Diagnostics tool.
- Fixed a bug that caused fgr files to break after the first save.
- Fixed a bug that allowed you to add PackedFiles from a PackFile into itself, effectively corrupting the added PackedFiles.
- Fixed quite a few annoyances of the diagnostics tool.
- Fixed quite a few bugs regarding parent-child widget relationships.
- Fixed a performance regression when saving PackFiles with the diagnostics tool enabled.
- Fixed a weird behavior when adding a folder to a PackFile while on MyMod mode.
- Fixed a bug that caused Global Search results to dissapear when double-clicking them.
- Fixed a bug that caused files with weird casing to be recognized as `Umknown` type and not being openable.
- Fixed a bug that caused RPFM to not start if you didn't have the msvcp140_1.dll file in your system.

## [2.2.1] - 2020-09-26
### Changed
- The diagnostics panel now it starts hidden if the diagnostics tool is disabled.

### Fixed
- Fixed a very time-specific CTD that caused crashes if you had an action going exactly 3 seconds after you edited a PackedFile.
- Fixed an issue where pasting `0` over float cells in a table didn't work.
- Fixed an issue that caused the autoupdater to leave files where it shouldn't.
- Fixed an issue that caused the dark theme to take some time to load if you started RPFM by doubleclicking on a PackFile.

## [2.2.0] - 2020-09-25
### Added
- Implemented Autosave.
- Implemented `Are you sure?` dialog when closing the main window.
- Implemented shortcuts to close/move to next/move to prev tab.
- Implemented the `Use Old Column Order (Keys first)` setting for people who prefer pre-2.x column order in tables.
- Implemented `Paste As New Row` for tables.
- Implemented `Diagnostics` panel (you need to enable it in the settings).
- Implemented `Copy Path` feature, to easely copy paths of files/folders in a PackFile.
- Implemented a not-very-optimal definition guesser for the PackedFile Decoder.
- Implemented a check to see if a table is outdated.
- Implemented `Resize tables on edits to content's size:` setting.
- implemented `Open MyMod Folder` feature.
- Implemented `Check Template Updates` feature.

### Changed
- Changed all fields that accept regex so it turns green or red depending if the regex is valid or not.
- Tweaked TreeView Colours to be more visible on light theme.
- Now the search field gets focused when opening the Global/Table Search pannels.
- Images now are shown in their original size, except if they are bigger than the current window.
- The view menu now uses checkboxes to show if a panel is visible or not.
- Newly added files are expanded automatically on the Treeview.
- Reorganized `Preferences` window.
- Updated Qt Bindings to 0.5.

### Fixed
- Fixed a CTD when starting RPFM without internet connection.
- Fixed a CTD caused by using Ctrl+Z after pasting a reference cell with the dependency checker enabled.
- Fixed a CTD caused by hitting Ctrl+Z too fast after importing a TSV.
- Fixed a CTD caused by hitting Ctrl+Z too fast after undoing a TSV Import.
- Fixed a CTD that caused a crash after pasting very specific float numbers.
- Fixed a CTD at the start if you didn't had vcruntime_140.dll somewhere in your system.
- Fixed a CTD when trying to generate a PAK for Rome 2.
- Fixed a CTD when trying to delete a schema definition.
- Fixed a CTD when trying to merge tables.
- Fixed a semi-random CTD when opening a PackFile from one of the submenus.
- Fixed a rare hang after performing certain actions that tried to use the table dependency data.
- Fixed a bug that caused `Load All CA PackFiles` to not work on games older than Warhammer.
- Fixed a bug that caused `Load All CA PackFiles` to hang the application until the files finish loading.
- Fixed an issue that caused `New Queek File` to create folders with files inside instead of just files.
- Fixed some harmless errors while merging tables.
- Fixed a couple of ordering bugs in the table column order logic.
- Fixed regex coloring on Global Search not working if you used the dark theme.
- Fixed a bug that caused image rescaling to not work as intended.
- Fixed a bug that caused schemas to be saved unordered.
- Fixed a bug that caused autosave to hang the program if autosave interval was set to 0.
- Fixed ghost items being created in the TreeView after using `Merge Tables`.
- Fixed a long-standing issue that triggered a harmless error when deleting a decodeable PackedFile from the TreeView.
- Fixed a bug that caused initialization of config folders to not work properly on debug mode.

### Removed
- Removed `Check Tables` feature, as it has been superseed by the new `Diagnostics` panel.
- Removed color coding for errors in tables.
- Removed `Command Palette`.

## [2.1.5] - 2020-08-22
### Added
- Implemented Autoupdater.

## [2.1.4] - 2020-08-15
- For this update and older ones, check the release page.

[Unreleased]: https://github.com/Frodo45127/rpfm/compare/v2.5.3...HEAD
[2.5.3]: https://github.com/Frodo45127/rpfm/compare/v2.5.2...v2.5.3
[2.5.2]: https://github.com/Frodo45127/rpfm/compare/v2.5.1...v2.5.2
[2.5.1]: https://github.com/Frodo45127/rpfm/compare/v2.5.0...v2.5.1
[2.5.0]: https://github.com/Frodo45127/rpfm/compare/v2.4.3...v2.5.0
[2.4.3]: https://github.com/Frodo45127/rpfm/compare/v2.4.2...v2.4.3
[2.4.2]: https://github.com/Frodo45127/rpfm/compare/v2.4.1...v2.4.2
[2.4.1]: https://github.com/Frodo45127/rpfm/compare/v2.4.0...v2.4.1
[2.4.0]: https://github.com/Frodo45127/rpfm/compare/v2.3.4...v2.4.0
[2.3.4]: https://github.com/Frodo45127/rpfm/compare/v2.3.3...v2.3.4
[2.3.3]: https://github.com/Frodo45127/rpfm/compare/v2.3.2...v2.3.3
[2.3.2]: https://github.com/Frodo45127/rpfm/compare/v2.3.2...v2.3.2
[2.3.1]: https://github.com/Frodo45127/rpfm/compare/v2.3.0...v2.3.1
[2.3.0]: https://github.com/Frodo45127/rpfm/compare/v2.2.1...v2.3.0
[2.2.1]: https://github.com/Frodo45127/rpfm/compare/v2.2.0...v2.2.1
[2.2.0]: https://github.com/Frodo45127/rpfm/compare/v2.1.5...v2.2.0
[2.1.5]: https://github.com/Frodo45127/rpfm/compare/v2.1.4...v2.1.5
[2.1.4]: https://github.com/Frodo45127/rpfm/compare/v2.1.3...v2.1.4
