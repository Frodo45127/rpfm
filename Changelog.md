# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project doesn't adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

If you're looking for the changes included in the latest beta (against the latest stable version), check the unreleased section.

## [4.3.1]
### Fixed
- Fixed CTD on save after editing a VMD or WSModel file.

## [4.3.0]
### Added
- Implemented basic CUF support (lib only).
- Implemented "Add RPFM as a Tool in Runcher" button in settings.
- Implemented support for open specific files in Packs when opening Packs through command line.
- Implemented support for secondary mod paths (like the one in Runcher).
- Implemented support for multi-level lookups.
- Implemented lookup support for filters.
- Implemented Warhammer 3 support in the Unit Editor.
- Implemented "Invalid Key on Lua Script" diagnostic.
- Implemented automatic translation of entries with the same value in the Translator.
- Implemented automatic translation of entries with the same value as a vanilla entry in the Translator.
- Implemented automatic translation using google when first clicking on an untranslated line.
- Implemented support for Pharaoh's Dynasties update.

### Changed
- Loose files in /data are now accessible as dependencies.
- Menus are now single-columns on Windows, like they've always been in Linux.
- Columns with lookups are now a bit bigger than the rest to account for the lookups.
- Changelog should now automatically open after an update.
- Autosaves are only triggered if the pack has been marked as "edited".
- Updated minimal rust version to 1.70.
- Temporarilly disabled the 3d renderer due to an unfixed memory leak.

### Fixed
- Fixed "Import from AK" not working due to a misplaced index.
- Fixed incorrect icons in anim, font and groupformation files.
- Fixed CTD when using the faction painter with a faction with no screen name.
- Fixed lookups not getting loc data from parent packs.
- Fixed parent packs not properly initializing their loc data under specific circustances.
- Fixed debug update schema function not properly cleaning invalid schema reference data.
- Fixed issues when processing Empire and Napoleon AK xml files.
- Fixed rare CTD when updating a table.
- Fixed incorrect path being set for automatically detected games.
- Fixed missing error message about the 'Rescue PackFile' feature.
- Fixed environment files being optimized away from map packs by the optimizer.
- Fixed "Update schema from AK" failing to find relations specified in TWaD_relationships when updating schemas.
- Fixed unit editor's window being too big and not scrollable.
- Fixed unit editor failing to load variantmesh files.
- Fixed error in the 3d viewer causing models to not load the rigidmodel editor.
- Fixed Drazhir lazyness on updates (may not work).
- Fixed windows not opening the changelog due to missing associations with md files.
- Fixed a double update triggering when trying to restart RPFM after an update.
- Fixed a lot of issues related to selection being broken in the Translator.
- Fixed a bug where sometimes the rigidmodel 3d renderer would fail to find textures.
- Fixed Rome 2 Build Startpos process not including victory objectives in the startpos.
- Fixed "Copy from Source" being a bit broken in the Translator.
- Fixed Pharaoh's AK path not being found automatically.

## [4.2.7]
### Added
- Implemented "Build Startpos" support for all games since Rome 2 (except Warhammer 1).

### Changed
- Campaign list in "Build Startpos" now has a consistent order.
- Table profiles are now game specific (if you have profiles, you need to go to "Game Selected/Open RPFM's Config Folder", open the table_profiles folder and move each json to the subfolder of the game you want. If it's not created, create a new profile and RPFM should initialize said folder).

### Fixed
- Fixed Build Startpos not properly reporting when a startpos generation failed.
- Fixed CTD when importing a folder from the Assembly Kit.
- Fixed tables in wrong folders when importing their folder from the Assembly Kit.
- Fixed factions tables not getting colour fields in some of their RGB fields.
- Fixed "Open Containing Folder" not working.
- Fixed "New Quick File" sometimes failing on tables if the db folder is uppercased.
- Fixed 99 limit on i64 cells.
- Fixed Sequence cells not correctly converting to/from TSV files.

## [4.2.6]
### Added
- Implemented "Multi-Folder FilePicker" feature.
- Implemented "Model Renderer" for RigidModels ranging from Rome 2 to Warhammer 3 (by [@mr-phazer](https://github.com/mr-phazer)).

### Fixed
- Fixed i64 columns not saving their value on focus out.
- Fixed decompression failing in vanilla files.

## [4.2.5]
### Added
- Implemented setting to allow export tsv to export with the alternative (as seen in dave) column order.
- Implemented "Update Anim Ids" feature.

### Changed
- Table filters now replace jumplines with or when the filter triggers, not when the text is written.
- "Build Startpos" can be now used with all games since Shogun 2, but it has only really been tested with Warhammer 3, and only Warhammer 3 and Troy have the schema parts needed for it to work.
- Refined diagnostics panel.
- Importing tsv locs from the CLI should no longer require an schema.

### Fixed
- Fixed CTD after using "Import from Assembly Kit" in decoder.
- Fixed undo not working on filters.
- Fixed dbg message when opening files.
- Fixed filters not triggering when adding files from another Pack.
- Fixed filters not triggering when adding files from AnimPacks.
- Fixed "Export TSV" not working on dependencies.

## [4.2.4]
### Added
- Implemented "Build Startpos" feature under Special Stuff. Only for Warhammer 3.
- Multiline text is now merged in one line with OR if you paste it in a table's filter.
- Certain Assembly Kit tables can now be imported into a working table from the dependencies view.
- New setting: "Ignore Game Files in Assembly Kit". Disabling it (default behavior) loads all tables from the Assembly Kit in the dependencies panel. Enabling it only loads those that are not in the game files (old behavior).

### Changed
- Reduced default size of the diagnostics panel.
- start_pos_xxx tables can now be created in RPFM if the schema has a definition for them.

### Fixed
- Fixed build errors of the cli due to steamlocate update.
- Fixed art set id not saving when edited in a portrait settings file.
- Fixed RPFM not correctly invalidating the dependencies cache after an update.

## [4.2.3]
### Fixed
- Fixed dark-theme error on linux builds.
- Fixed Pack not saving after adding a file, then trying to extract it.
- Fixed decoding error in anim tables after adding a new row.
- Fixed "Merge files" deleting the destination file if it was one of the source files.

## [4.2.2]
### Added
- Implemented commands to manipulate Pack Dependencies through the CLI.

### Fixed
- Fixed Arch PKGBUILD not building due to a dependency name change.
- Fixed broken pack sorting logic.
- Fixed CTD when a CA_VP8 file has 0 frames.

## [4.2.1]
### Fixed
- Fixed some ca_vp8 videos (mainly Pharaoh ones) not opening correctly.
- Fixed deleting filters not deleting them correctly, leaving tables stuck in a partially filtered state.
- Fixed incorrect message shown when updating.

## [4.2.0]
### Added
- Implemented "Translator" tool, with support for all games.
- Implemented support for "HlslCompiled" files (lib-only).
- Implemented support for Pharaoh.
- Implemented support for GroupFormation files for:
    + Rome 2
    + Shogun 2
- Implemented "Update Manager", and merged all "Check XXX Updates" buttons in it.

### Changed
- BMD files should no longer have a red cross as icon.
- BMD Vegetation now have a particular icon.
- Updated optimizer warning.
- The mysterious error non-error message when generating the dependencies cache is now less mysterious.
- ESF Editor has been disabled and hidden behind a setting due to bugs.
- The "move table" error message should be clearer now.

### Fixed
- Fixed a bug in the compression logic that caused errors when compressing/decompressing.
- Fixed checkbox filtering working on reverse.
- Fixed functions that merged packs overwriting files in the wrong order.
- Fixed read-only tables still allowing to edit checkbox columns.
- Fixed table row counter not updating under certain situations.
- Fixed many pack features not working for packs open in a special way.
- Fixed Empire not counting "movies.pack" as a vanilla file.
- Fixed incorrect order in default game combobox.
- Fixed CTD when saving a CA_VP8 file with no frames.
- Fixed frg files not being properly identified when outside of the animations folder.
- Fixed outdated error message pointing to "preferences".
- Fixed crash logger setting the app name incorrectly.
- Fixed CTD when changing the type of a Pack through the CLI.
- Fixed logger error spam due to a missing check.
- Fixed "Add to Pack" operation decoding files they shouldn't decode.
- Fixed certain tables hanging when you click on their tab due to having too many icons.
- Fixed "Update schema from assembly kit" feature missing localisable fields when a localisable fields file existed.

## [4.1.3]
### Added
- Implemented Anim Fragment Battle view.
- Implemented support for Troy's Anim Fragment Battle files.
- Implemented Global Search&Replace support for Anim Fragment Battle files.
- Implemented diagnostics support for Anim Fragment Battle files.
- Implemented path checking diagnostics for Portrait Settings files.
- Implemented support for loading animation index files as lookup data for Anim Fragment Battle files.

### Changed
- Anim Fragment Battle files now use their own view instead of the debug one.
- Optimized code to check if a file/path exists as a file in a pack.
- Diagnostics for Portrait Settings files and Anim Fragment Battle files are now enabled by default.
- Performing a diagnostic check should no longer disable the entire window.
- More optimizations to the diagnostic code.

### Fixed
- Fixed subtables not saving correctly.
- Fixed type conversion functions breaking subtables.
- Fixed AnimFragmentCampaign bins being marked as AnimFragmentBattle bins, causing RPFM to give them an incorrect icon and error out when clicking on them.
- Fixed error when opening certain flyer's and cavalry AnimFragmentBattle files.
- Fixed ESF files for Shogun 2 breaking after saving their pack.
- Fixed "Field with Path not found" diagnostic not working.
- Fixed diagnostics sometimes missing recent changes in the views.
- Fixed double-clicking on a portrait settings diagnostic not selecting the diagnosed field.
- Fixed diagnostics not being executed in files that have not yet been open.
- Fixed a situation that would cause an "only open files" diagnostic to trigger a partial full diagnostic of a pack instead.
- Fixed CTD when trying to use Global Replace over an entire Portrait Settings file.

## [4.1.2]
### Added
- Improved formatting of the paths used for icons in the table column tooltips.
- In tables, the column tooltips now say where do lookups get their data.
- General optimisations to the dependencies and diagnostics systems.
- Implemented support for the following Text files:
    - ".bat"
    - ".cco"
    - ".environment_group"
    - ".environment_group.override"
    - ".fbx"
    - ".fx"
    - ".fx_fragment"
    - ".h"
    - ".hlsl"
    - ".kfc"
    - ".kfe_temp"
    - ".kfl_temp"
    - ".kfcs_temp"
    - ".ktr"
    - ".ktr_temp"
    - ".mvscene"
    - ".py"
    - ".sbs"
    - ".shader"
    - ".xml_temp"

### Fixed
- Fixed multiple issues related with lookups not showing up or not updating on certain columns.
- Fixed locs in the open pack not being used correctly for searching for lookup data.
- Fixed a bug that caused certain files in the open pack to not be used for reference data if something in their path had uppercases.
- Fixed icon fields not resolving icons when the provided path used \\ instead of / as separator.
- Fixed icon search not working correctly when the field can be searched in multiple paths.
- Fixed lookups not working correctly when the lookup pointed to multiple columns.
- Fixed actions that open files and scroll them failing to do the scroll part.

## [4.1.1]
### Added
- Implemented local schema patches.

### Changed
- Highly optimised pack saving speed.
- Increased default width for numeric fields in tables.

### Removed
- Removed uploading patch schemas.

### Fixed
- Fixed CTD when trying to replace a db table global search match.
- Fixed compression errors while saving causing the entire pack to fail to save and be left broken on disk.
- Fixed lookups pointing to the same table they're shown in not working.
- Fixed lookups not updating when pasting.

## [4.1.0]
### Added
- Implemented warning when the autosaves folder surpasses 25GB.
- Implemented table lookups (like in DAVE).
- Implemented table profiles.
- Implemented support for Atlas files.
- Implemented custom view for UnitVariant files.
- Implemented Global Search & Replace support files of the following types:
    - Atlas
    + Portrait Settings
    + RigidModel
    + Unit Variant
    + Unknown
- Implemented downloader for Empire and Napoleon's AK-like XML files.
- Implemented parser for Empire and Napoleon's AK-like XML files.
- Implemented support for all the features that required the Assembly Kit for Empire and Napoleon.
- Implemented icon viewer in Portrait Settings views.
- Implemented icon view in table fields that reference an image.
- Added button to toggle regex on table filters (enabled by default).

### Changed
- Optimized Dependencies TreeView UI loading process.
- Optimized Global Search UI loading process.
- Optimized Pack TreeView UI loading process.
- Optimized the UI's file loading logic.
- Optimized table's UI loading logic.
- Optimized file-loading logic.
- Improved Crash recording to avoid duplicate reports.
- Improved table filter performance.
- DB Table automatic cell size now can vary between 60 and 800px, so you don't get too small columns, nor extremely big columns.
- The Search Source check for global search is now remembered between sessions.
- Global search matched text is now trimmed to focus on the part of the match.
- Global search's row column is now show right after the matched text.
- I64 and OptionalI64 fields in tables now use spinboxes, like the other numeric fields.
- Generate Ids now supports I64 ids when all selected fields admit i64 numbers.
- Heavely optimized code for finding files in the dependencies.
- Compressing packs no longer requires 7zip.

### Removed
- Removed 4.0 warning message on first start.
- Removed unused SQLite backend.

### Fixed
- Fixed missing window icon when using RPFM on linux under Wayland.
- Fixed Global Search Trees not properly animating.
- Fixed dependencies for files being queried twice on file open.
- Fixed CTD when applying a rewrite selection to a db column of certain types.
- Fixed multiple issues with numeric values in ESF files doing weird things.
- Fixed old ESF files with large 00 paddings at the end not opening.
- Fixed hang when using global replace on an open table with filters applied.
- Fixed portrait settings view not clearing properly on reload.
- Fixed CTD when using global replace under specific circustances.
- Fixed a ton and a half of inconsistencies on how loc data is loaded for references/lookups.
- Fixed Go To Loc not working with AK sources.
- Fixed a bug that caused dependencies cache to be ignored and regenerated when re-opening a file/switching tabs.
- Fixed global replace replacing all matches on the same line/cell if there were more than one.
- Fixed a myriad of inconsistencies when using different search modes.
- Fixed a bug where ESF files may lost numeric data when dealing with very large integers.
- Fixed Portrait Settings view not changing variant names properly.
- Fixed dependencies cache failing to generate for Thrones due to exe name change.
- Fixed decryption logic failing silently to decrypt half the files.
- Fixed open empty tabs with no name when a file fails to open after the tab is created.
- Fixed WH2 dependencies ignoring audio_base.pack due to a typo.
- Fixed "All" diagnostic filter checkbox toggling instead of checking all filters.
- Fixed resizable labels with images doing weird things when they are wider than their container.
- Fixed duplicated keys diagnostic not working on loc tables.
- Fixed paste commands pasting in the wrong cells when there are hidden columns in the table.


## [4.0.22]
### Fixed
- Fixed paths magically not working unless they're cannonicalized.

## [4.0.21]
### Fixed
- Fixed crash on start.

## [4.0.20]
### Added
- Implemented experimental support for opening BMD/BMD Vegetation/Bin files (lib-only).
- Implemented experimental support for opening cs2.parsed files (lib-only).
- CSC files can now be open in the ESF editor (not very user-friendly, but it works).
- Added counter under each diagnostic filter button.
- Added "Ignore..." options for diagnostics.
- Import/Export MyMod now has default showtcuts.

### Fixed
- Fixed portrait settings with specific names not being openable.
- Fixed animpacks causing RPFM to fail to save.
- Fixed infinite loop when renaming references.
- Fixed lost focus when opening certain dialogs.
- Fixed filter dropdowns on tables sometimes having the wrong table order.
- Fixed packs in bin format not showing up as part of the content packs.
- Fixed CTD when cleaning up diagnostics results if the table is broken.
- Fixed CTD when opening subtables.

## [4.0.19]
### Added
- Optimizer can now remove xml files from the prefab folder.

### Fixed
- Fixed `Add from Pack` not working when clicking on folders.
- Fixed `Pack Map` having multiple issues when tilesets were involved.
- Fixed optimizer optimising away rivers from maps.
- Fixed optimizer optimising away files in very specific paths that it shouldn't touch.
- Fixed parent dependencies not getting properly invalidated on rebuilds.
- Fixed dependencies not getting properly invalidated on settings path change.
- Fixed dependencies files getting closed when creating a new pack.

## [4.0.18]
### Changed
- Autosaves are once again limited to mods and movies.

### Fixed
- Fixed a bug that caused autosaves to be made of CA Packs on Windows.
- Fixed a bug that caused `disable autosaves for this Pack` setting to not work.

## [4.0.17]
### Changed
- Autosaves are no longer triggered for vanilla packs, regardless of their type.

### Fixed
- Fixed a variety of bugged behavior/crashes when deleting something from the Pack TreeView.

## [4.0.16]
### Fixed
- Fixed trailing zeroes on tables.

## [4.0.15]
### Added
- RPFM CLI can now export schemas as JSON.
- RPFM CLI can now import/export Portrait Settings files as JSon.
- Added icon to column title on columns with description (thanks [@SWarrener](https://github.com/SWarrener)).
- Implemented `Delete empty folders after moving/deleting its contents` setting.
- Implemented button to enable diagnostics to check against AK-only tables (may cause false positives).
- Implemented support for custom dark themes not overwritten by updates.
- Implemented support for custom icons not overwritten by updates.
- Implemented Assembly Kit path autodetection (only for Steam installations).
- Implemented `Pack Map` Special feature for Warhammer 3.

### Changed
- Improved grammar on "Are you sure?" dialogs (thanks [@gqqnbig](https://github.com/gqqnbig)).
- Schema patches can only be send if you filled the explanation field.
- The `Full Path Move Enabled` checkbox when renaming now remembers its status from the last time you used it.
- `F2` now triggers the Rename/Move dialog.
- Table cells with extremely big or extremely small numbers should no longer use scientific notation.

### Removed
- Removed `Other` Pack Type button (it was unused since 4.0 and caused CTDs).

### Fixed
- Fixed grammar on diagnostic message.
- Fixed "Paste" not pasting correctly on tables under specific situations.
- Fixed autosave not actually autosaving due to a myriad of errors and bugs.
- Fixed missing window title on merge tables dialog.
- Fixed merge tables only trying to merge open tables, ignoring the rest of the selected ones.
- Fixed incorrect pack type when opening multiple packs together (like when merging packs).
- Fixed videos on dependencies having the convert buttons clickable.
- Fixed help command on rpfm cli `merge` command failing on runtime.
- Fixed .gitignore not getting ignored when creating a MyMod with git support.
- Fixed alternative shortcuts not working.
- Fixed optimizer not deleting xml files in map folders.
- Fixed new MyMods having an incorrect path for the TW Autogen stuff.

## [4.0.14]
### Fixed
- Fixed CTD when opening RPFM without having steam installed.

## [4.0.13]
### Added
- RPFM will now autodetect games installed through steam if a path for them has not been specified.

### Changed
- ESF editor now use the updated filterable tree.
- Checkboxes should be more visible when using the dark theme.

### Fixed
- Fixed rare CTD when merging files.
- Fixed barely visible github icon.
- Fixed Shogun 2 AK not being properly used when generating dependencies.
- Fixed animpacks saved not opening on Assed.

## [4.0.12]
### Fixed
- Fixed CTD when extracting files.

## [4.0.11]
### Changed
- Removed a few unused files from releases to reduce their size.

### Fixed
- Fixed CTD when trying to load a definition in the decoder.
- Fixed multiple decoder bugs.
- Fixed autosaves being done on packs they shouldn't be done.
- Fixed weird issues when deleting and readding a filter.
- Fixed import from dependencies importing files it shouldn't.
- Fixed open with external tools not working on dbs.
- Fixed open with external tools corrupting tables when used incorrectly.
- Fixed treeview ordering failing in very rare situations.

## [4.0.10]
### Fixed
- Fixed a bug that caused TSV tables with colour columns to receive a wrong value in their colour columns.

## [4.0.9]
### Changed
- Replaced diagnostic tool icons to make the checkable buttons more visible.

### Fixed
- Fixed a bug compression will fail if the Pack had 2 or more compressible files.

## [4.0.8]
### Fixed
- Fixed a bug where compressing an uncompress file without a 7zip path set could lead to a corrupted Pack.
- Fixed a bug that caused RPFM to generate PFH6 Packs with part of the header missing.

## [4.0.7]
### Fixed
- Fixed false positives of path matches when creating a new folder.
- Fixed false positives of path matches when renaming/moving folders.
- Fixed schema patch importer not saving schema after an import.
- Fixed incorrect path parsing in animpacks.
- Fixed CTD when using global replace on Text matches.
- Fixed optimizer and tool save not actually saving the optimized files.

## [4.0.6]
### Changed
- "Include Parent Folder when Adding a Folder" setting is now enabled by default.

### Fixed
- Fixed diagnostics being able to trigger when no game path has been configured, leading to false positives.
- Fixed (hopefully) error about adding files to an animpack being detected as modified and failing to save.
- Fixed missing tooltip in "Include Parent Folder when Adding a Folder" setting.
- Fixed CLI crashes failing to be logged.
- Fixed incorrect error message after renaming an open file.
- Fixed renamed previews losing the "preview" status.
- Fixed "Data is Compressed" checkbox not being enabled when loading a Pack with compressed files.

## [4.0.5]
### Added
- Added a setting to add the base folder when using "Add From Folder".

### Changed
- When drag&droping to the db folder, you'll now receive a warning.

### Fixed
- Fixed rare CTD when marking items as modified
- Fixed mods with uppercased tables crashing the game.
- Fixed table UUID failing to generate on new tables.
- Fixed error after compressing packs.

## [4.0.4]
### Fixed
- Fixed mods with uppercased tables crashing the game.
- Fixed CTD when hitting Delete or Clone on variant list of Portrait Settings when nothing is selected.

## [4.0.3]
### Fixed
- Fixed error when successfuly updating the schemas after a cleanup.
- Fixed tables not being searchable when using "Load All CA Packs".
- Fixed "Add from Folder" not adding the files correctly.
- Fixed "Load All CA Packs" autosaving when it shouldn't.
- Fixed CTD when clicking on bnk files.
- Fixed small issue regarding Pack Encoding order.
- Fixed multiple instances of certain settings related with file encodings not working.

## [4.0.2]
### Fixed
- Fixed parent mods not loading properly from /data.

## [4.0.1]
### Fixed
- Fixed corruption on tables due to incorrectly merged colour columns.
- Fixed Test definition not working.
- Fixed decoder duplicating definitions.
- Fixed incorrect diagnostic marking for optional integer columns.
- Fixed extract tables not actually extracting tables as tsv.

## [4.0.0]
### Added
- Fully rewritten RPFM CLI tool.
- Fully rewritten RPFM lib to be a proper library, fully independant from the UI.
- Large performance improvements.
- Ram usage improvements.
- Split "rpfm_lib" into "rpfm_lib" and "rpfm_extensions".
- UI largely refactored from hardcoded versbose UI to a more template-based iconic UI.
- Added ArchLinux PKGBUILD for rpfm-bin (precompiled stable releases) and rpfm-git (latest git builds).
- Added rpfm_lib, rpfm_extensions and rpfm_cli to crates.io.
- Added diagnostics over Portrait Settings files.
- Added message widget on main window (used by a few messages for now).
- ESF Editor is out of beta and can be used without changing settings.
- Implemented support for Anim Fragments for Wh2, Wh3 and 3k.
- Implemented support for Anims Tables for Wh2.
- Implemented support for creating Portrait Settings files
- Implemented support for Gif files.
- Implemented support for Matched Combat tables for Wh2, Wh3 and 3k.
- Implemented support for MP3 files.
- Implemented support for Portrait Settings files
- Implemented support for SoundBanks (lib-only, incomplete and unusable for now).
- Implemented support for TWC (character saves) files.
- Implemented support for Warhammer 3, Linux version.
- Implemented support for Drag&Drop opening Packs.
- Implemented support for Drag&Drop moving files/folders on an open Pack.
- Implemented support for "Global Replace" for Text files.
- Implemented support for the following Text files:
    - ".code-snippets"
    - ".code-workspace"
    - ".css"
    - ".js"
    - ".kfa"
    - ".kfe"
    - ".kfl"
    - ".kfsl"
    - ".kfp"
    - ".kfcs"
    - ".twui"
    - ".material"
- Implemented "Extract from Dependencies" feature.
- Implemented "Live Export" feature (for scripting).
- Implemented "Not" or "!" checkbox for table filters.
- Implemented "Reload Style-Sheets" debug option.
- Implemented Status Bar icons.

### Changed
- "Add" now automatically adds tables as TSV if a valid schema is present.
- "Extract" now automatically extracts tables as TSV if a valid schema is present.
- "New MyMod" dialog now offers a more granular configuration.
- Autosaves are now per-pack, with the date they're made as name.
- Dark Theme now works on linux.
- Disabled "Path invalid" diagnostic by default due to performance issues.
- Double-clicking the file header now causes the TreeViews to scroll to the files, if they're not filtered out.
- Extracted dark theme's stylesheet to an editable "dark-theme.qss" plain text file.
- File/Folder icons updated, with more file-type specific icons.
- Global Search UI now use a merged TreeView for results instead of one per file-type.
- Load All CA Packs is now disabled if you have dependencies generated.
- Minimal Rust version updated to 1.64.
- Reimplemented remains of the old Tips system into a proper Quick Notes system, integrated in the Packs similar to normal notes.
- Rename is now Rename/Move and has been refactored to allow moving files and folders.
- Replacing a Global Search match no longer closes the expanded results.
- Reworked how schema patches are managed internally, so they actually affect more than simple table views.
- Updated dependencies.

### Fixed
- Fixed "Generate Loc Data" generating incorrect loc keys for multikeyed tables.
- Fixed "Rename References" not working properly when used on references.
- Fixed AnimPacks from dependencies not importing files correctly.
- Fixed CTD when reloading ESF files.
- Fixed CTD when deleting files.
- Fixed CTD when renaming files.
- Fixed faction painter failing to load the correct localisation data for each faction.
- Fixed files not saving if edited right after importing them from dependencies.
- Fixed focus being lost from focused widgets when an operation disables the main window.
- Fixed issues with incorrect localisation dependencies under Linux.
- Fixed missing icons on text view and text settings.
- Fixed operations that try to guess a loc key from multikeyed tables.
- Fixed schema having invalid reference values.

### Removed
- Removed local/remote tips system.
- Removed Mass-Import and Mass-Export TSV.
- Removed rpfm.exe, as it's no longer needed for program restarts.
- Removed settings related to disabling automatic git repo and project files when generating a MyMod.

## [3.0.16]
### Changed
- Improved by more than 50% the time it takes to populate the dependencies and the pack treeviews.
- Improved boot times.

### Fixed
- Fixed a CTD when trying to open multiple instances of RPFM.

## [3.0.15]
### Added
- Implemented "Copy to filter value" feature.

### Fixed
- Fixed inconsistencies on Copy as LUA Table feature.
- Fixed a ton of performance issues in "Copy as LUA Table" feature.
- Fixed CTD on certain operations when using characters outside the ascii range.

## [3.0.14]
### Fixed
- Fixed diagnostic tool's false positives due to checks against tables in the AK.
- Fixed incorrect translation on table filters.

## [3.0.13]
### Changed
- Improved performance of "Copy Table as Lua Table" feature.
- Renamed "New Queek File" to "New Quick File" as people don't understand what this feature is.

### Fixed
- Fixed missing float precision on copy.

## [3.0.12]
### Changed
- Updated PKGBUILD so it "should" work.
- Trim search results so text searches are shown without tabulations or spaces before or after.
- Text results should open the respective files and scroll to the proper line on double-click.
- `Copy as Lua table` now copies entries with duplicated keys outside the main table.

### Fixed
- Fixed incorrect sublime project file being generated on new MyMods.
- Fixed `Copy as Lua table` removing the final comma from the wrong row.

## [3.0.11]
### Changed
- Updated paths for TW Autogen support (only new MyMods).
- Implemented a few microoptimizations that may improve speed and responsiveness on operations that require access to dependencies cache or table references.

### Fixed
- Fixed CTD when opening notes.
- Fixed issue where diagnostics about references to AK-only tables were missed.

## [3.0.10]
### Fixed
- Fixed issue with wrong language packs being loaded in dependencies in certain games.
- Fixed issue with last 3K update breaking dependencies cache generation.

## [3.0.9]
### Fixed
- Fixed table `allied_recruitment_units_tables` being locked instead of `allied_recruitment_core_units_tables` after the last WH3 Patch.

## [3.0.8]
### Added
- Implemented setting to disable automatic Git Repos and Config files on new MyMods.

### Fixed
- Fixed slowdowns due to a logging file getting into a logging frenzy.
- Fixed crash due to people ignoring the big warning that says "Do not change the table folders name".
- Fixed rare crash due to a specific interaction between backend and frontend.

## [3.0.7]
### Fixed
- Fixed `Generate Loc Data` only generating one row in multikeyed tables.
- Fixed paste not working properly on colour columns, again.

## [3.0.6]
### Added
- Implemented `Find References` feature.
- Implemented `Clear Dependencies Cache` feature.
- Implemented `Generate Loc Data` feature.
- Implemented support for `TW Autogen` installation for MyMods.

### Changed
- Diagnostics should not run now if the diagnostic panel is closed.
- Dependencies cache will now load part of it's data only when triggering a diagnostic search, as that data is not used otherwise.
- Implemented some microoptimizations that should help to get down the memory usage.
- When creating a MyMod, a git repository will automatically be created in the MyMod folder.
- When creating a MyMod, on supported games (currently, only WH3) config files will be created in the MyMod folder for VSCode and Sublime Text to make TW Autogen automatically work for them. Read TW Autogen docs for more info about it: [https://github.com/chadvandy/tw_autogen](https://github.com/chadvandy/tw_autogen).

### Fixed
- Fixed a CTD when pasting invalid data on a colour column.
- Fixed `Check Message Updates` setting not working.
- Fixed hidden columns not getting ignored when performing any cell-editing operation on tables.
- Fixed faction painter being broken for games other than WH3.
- Fixed unit editor.
- Fixed `Go To Loc` feature not working on multi-keyed tables.
- Fixed multiple issues with the CAMV<->IVF conversions.
- Fixed fallback behavior when loading vanilla data so it no longer loads mods.
- Fixed hang on very specific failures when importing a MyMod.
- Fixed hang on Global Search's `Replace All` feature.
- Fixed Global Search results being lowercased when they shouldn't.
- Fixed Global Search's replace not working when not searching in case-sensitive mode under certain situations.

## [3.0.5]
### Fixed
- Fixed crash at boot due to libgit2 update.

## [3.0.4]
### Added
- Implemented Schema Patcher.
- Implemented `Value cannot be empty` diagnostic.

### Changed
- Floats on DB Tables should now support up to 4 decimals, instead of 3.

### Fixed
- Fixed a bug that caused RPFM to use an incorrect header when saving PackFiles as Mod Type for Warhammer 3.

## [3.0.3]
### Changed
- Improved error messages under certain circustances.
- RPFM no longer adds rpfm-specific files to vanilla packs.
- RPFM no longer adds version header to version 0 tables.

### Fixed
- Fixed a bug that caused some colour fields to be encoded incorrectly.
- Fixed a bug that caused alternative 0 version definitions to not work.
- Fixed a bug that caused the decoder version to not change on load.

## [3.0.2]
### Fixed
- Fixed a bug that caused an incorrect PFHX version being set on PackFiles.

## [3.0.1]
### Fixed
- Fixed incorrect game selected set when opening packfiles.
- Fixed saving error when saving packs with specific tables.

## [3.0.0]
### Added
- Implemented support for Warhammer 3.
- Implemented new `Messages` feature.
- Implemented small search history on Global Search.
- Implemented support for F64 fields.
- Implemented support for RGB fields.
- Implemented support for banned tables.
- Implemented diagnostic to detect banned tables.
- Implemented partial support for .bin anomfragments.
- Implemented support for grouped colour columns.

### Changed
- Rewrite selection now accepts uppercased versions of the replaced characters.
- Schemas should load a bit faster on slow disks.
- Extract commands of the cli now autoexport tables.
- Updated all games icons (thanks to Jake Armitage).
- `Dependencies` and `Load all CA PackFiles` should now only load your selected language files and ignore the rest of language files.
- Updated schema version to v4 to support the new field types.
- Updated faction painter to work with Warhammer 3.
- Updated dependencies.

### Fixed
- Fixed OOM bug on Global Search related with text files.
- Fixed `Go To` commands not expanding on the views on the left the files they open.
- Fixed text files not being marked as modified in the TreeView.
- Fixed text files showing the last line and not the first one on opening.
- Fixed missing templates on linux release builds.
- Fixed a bug on linux builds that caused the Unit Editor to not load.
- Fixed the cli not working properly with Troy.
- Fixed the decoder weirdly falling to report the errors on v0 tables.
- Fixed a bug that caused weird stuff to happen in the first right-click of a table.
- Fixed multiple issues related with read-only tables not being properly locked.

## [2.6.6]
### Added
- Implemented Row Counter for tables.
- Implemented `Disable PackedFile Previews` setting.
- Implemented `Variant Editor` subtool (within `Unit Editor`).

### Changed
- Tools now require you to generate the dependencies cache with the assembly kit files included.
- Optimizer now require you to generate the dependencies cache with the assembly kit files included.

### Fixed
- Fixed multiple issues related with Arch PKGBUILD and UI files.
- Fixed `Copy Unit` button not working on the Unit Editor.
- Fixed a number of minor issues on the Unit Editor.
- Fixed CTD when merging tables with special symbols on their name.
- Fixed CTD when opening a PackFile from a submenu, then quickly opening that submenu again.
- Fixed -A option on the CLI not working as expected.

## [2.6.5]
### Added
- RPFM CLI can now export schemas to XML.

### Changed
- Replaced hashing dependencies with a more performance-friendly option.
- Dependencies Cache load error is now more... less horrifing.

## [2.6.4]
### Changed
- Optimizer now also removes ANY unchanged file (relative to Parent/Vanilla Packs).
- Unit editor can now save data.

### Fixed
- Fixed CTD when hitting `Launch Game Selected` with Troy as Game Selected.
- Fixed CTD when a rare read IO error is detected while trying to import files.
- Fixed CTD (AGAIN, this time properly) when executing a global replace.
- Fixed CTD (hopefully, I didn't managed to reproduce this one) when starting the program.
- Fixed CTD when there are wrongly named files in the locale folder.
- Fixed CTD when the autosave folder is missing/not readable.
- Fixed CTD when trying to decode a string with very specific caracteristics.
- Fixed CTD when decoding a CA_VP8 fails.
- Fixed CTD when saving certain rigids fails.
- Fixed CTD when, trying to import a MyMod, reading the MyMod folder fails.
- Fixed CTD when trying to optimize a PackFile and RPFM was in russian (by [@im-mortal](https://github.com/im-mortal)).
- Fixed optimizer failing to optimize away certain rows.
- Fixed a lot of typos (by [@im-mortal](https://github.com/im-mortal)).


## [2.6.3]
### Added
- Added russian localisation (by [@im-mortal](https://github.com/im-mortal)).

### Changed
- Optimizer now also removes duplicate/new/empty rows.
- Added a flag to ensure the subclasses lib is recompiled alongside RPFM.
- Updated compilation instructions (by [@im-mortal](https://github.com/im-mortal)).

### Fixed
- Fixed CTD when opening broken/incompatible AnimPacks.
- Fixed CTD when adding an empty folder.
- Fixed CTD when inputting a multibyte character (like a kanji) on the `New PackedFile` dialog (again).
- Fixed CTD when someone deliberately messes up the settings file.
- Fixed CTD when replacing an open rigidmodel with another which RPFM cannot open.
- Fixed (hopefully) a random CTD when RPFM needed to back data to the backend on mass..
- Fixed `column` filter combobox order changing on table reload.
- Fixed Faction Painter Tool adding entries for factions that don't need them.
- Fixed tables not changing after using a Tool.
- Fixed dependencies not always reloading correctly.
- Fixed ESF editor wiping out negative numbers (this time for good).
- Fixed Optimizer not properly removing certain rows containing floats (AGAIN).

## [2.6.2]
### Added
- Implemented log rotation.

### Changed
- Improved release build times.
- Improved release build script.
- Sorted faction list on Faction Painter tool.
- Updated compilation instructions.

### Fixed
- Fixed CTD on Global Search.
- Fixed CTD when opening certain Global Search results.
- Fixed CTD when files that shouldn't be set as modified were set as modified.
- Fixed CTD when you triggered two consecutive warnings on one of the tools.
- Fixed Grey Screen Of Death when trying to open a PackedFile without even downloading the schemas.
- Fixed dependency getter pulling files from Vanilla files instead of Parent files.
- Fixed missing icons in certain games in the Faction Painter.
- Fixed certain situations where RPFM could incorrectly pull data from a outdated dependency.
- Fixed Mass-Import TSV importing locs as DB tables.
- Fixed loc fields saving incorrectly in Unit Editor.
- Fixed key fields saving incorrectly in Unit Editor.
- Fixed ESF editor wiping out negative numbers.
- Fixed RON/JSON files generated by this tool not terminating in \n (POSIX Standard, by [@im-mortal](https://github.com/im-mortal)).

## [2.6.1]
### Fixed
- Fixed CTD on start on new installations.
- Fixed CTD when clicking on any of the tools.

## [2.6.0]
### Added
- Implemented warning when trying to rename a DB folder.
- Implemented CCD(CEO)/ESF/SAVE editor (EXPERIMENTAL).
- Implemented Dependencies View.
- Implemented Global Search Support for dependencies.
- Implemented `Import from dependencies`.
- Implemented `Tools` menu.
- Implemented `Faction Painter` tool.
- Implemented `Unit Editor` tool (EXPERIMENTAL, READ-ONLY FOR NOW).
- Implemented `Only For The Brave` alert for specially unstable builds.

### Changed
- Updated dependencies.
- Updated KTextEditor.
- RigidModel View updated to 0.8.2 (includes fixes for issues found in RPFM 2.5.4 regarding broken models).
- Rigidmodel error messages should now be more specific.
- TSVs to be imported no longer require to have all the columns of a table.
- TSVs to be exported now export using the column order you see in the UI.
- TSVs first and second rows have been swapped, to allow programs that expect the first row to be the column headers to actually not complaint and work.
- TSVs now store on their second row their original path, so it can be restored properly when using `Use original filename` on a MassImport.
- TSVs second row can now be marked with # so tools using them can be configured to ignore it.
- TSVs metadata now its contained in the first cell of the second row, split by `;`.
- Import/Export MyMods now import/export tables as TSV if they can.
- Import/Export MyMods now import/export notes and PackFile settings.
- When installing a PackFile, a save is now automatically done before the install, so the installed PackFile is always the most up-to-date.
- When using `Generate Ids`, now the value of the first cell is used as default value.
- `Rewrite Selection` {z} replacer now use the row number relative to the selection, not to the table itself.
- Reworked internal threads comms to make each action use their own comm channel (should fix multiple CTD).
- Improved logging logic to... actually work most of the time.
- Many Clippy-suggested corrections.

### Removed
- Removed `Templates` feature.

### Fixed
- Fixed multiple `Open ... Folder` actions hanging the program until you closed the explorer window.
- Fixed focus not being set to tables after double-clicking on a Global Search/Diagnostic result.
- Fixed very rare bug that caused RPFM to fail on reimport very specific TSV files.
- Fixed a bug that caused dependencies to not update properly on game selected change.
- Fixed multiple bugs that caused dependencies to become missing or unloaded from time to time.
- Fixed dependencies going away immediately after generating them.
- Fixed missing shortcuts in the view submenu.
- Fixed `Some("","")` references being incorrectly imported from the Assembly Kit.
- Fixed error when exporting a TSV if its parent folder doesn't exist.
- Fixed a CTD caused by opening a menu when a PackFile is being opened.
- Fixed a CTD when opening PackFiles with the `Check for missing table definitions` setting enabled and no writing permissions on RPFM's folder.
- Fixed a CTD when hitting `Replace` on the Global Search.
- Fixed a CTD when trying to load RPFM without the `Locale` folder.
- Fixed a CTD when opening the dependencies manager on certain games.
- Fixed a CTD when inputting a multibyte character (like a kanji) on the `New PackedFile` dialog.

## [2.5.4]
### Added
- Reworked `Game Selected` logic.
- Added `Assembly Kit` fields to settings.

### Changed
- Updated compilation instructions.
- Improved Game folders detection.
- Improved paths tab on settings.

### Fixed
- Fixed case sensitive checkbox on Global Search working on inverse.
- Fixed multiple issues while compiling on linux.
- Fixed update table not performing type updates correctly.
- Fixed decoder not picking up updated data after a table gets updated.
- Fixed lockups on `Rename references` (again).
- Fixed incorrect action getting triggered after double-clicking the `Dependencies Cache outdated` diagnostic.
- Fixed `Optimize PackFile` failing to optimize certain float values.
- Fixed a myriad of issues related to game selected, dependencies cache, diagnostics, and the linux ports of Total War games.

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
- Removed requirement of Assembly Kit for Generation of Dependencies Cache.
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
- Implemented `Game-Wide` cache (replaces the old PAKs).
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
- Fixed CTD that happened sometimes when trying to use the table decoder.
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
- Improved responsiveness during diagnostics checks.
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
- Fixed a bug that caused the `Delete` function on tables to not delete properly a row if it had hidden columns.
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
- Fixed a bug that caused Global Search results to disappear when double-clicking them.
- Fixed a bug that caused files with weird casing to be recognized as `Unknown` type and not being openable.
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
- Implemented `Copy Path` feature, to easily copy paths of files/folders in a PackFile.
- Implemented a not-very-optimal definition guesser for the PackedFile Decoder.
- Implemented a check to see if a table is outdated.
- Implemented `Resize tables on edits to content's size:` setting.
- implemented `Open MyMod Folder` feature.
- Implemented `Check Template Updates` feature.

### Changed
- Changed all fields that accept regex so it turns green or red depending if the regex is valid or not.
- Tweaked TreeView Colours to be more visible on light theme.
- Now the search field gets focused when opening the Global/Table Search panels.
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

[Unreleased]: https://github.com/Frodo45127/rpfm/compare/v4.3.1...HEAD
[4.3.1]: https://github.com/Frodo45127/rpfm/compare/v4.3.0...v4.3.1
[4.3.0]: https://github.com/Frodo45127/rpfm/compare/v4.2.7...v4.3.0
[4.2.7]: https://github.com/Frodo45127/rpfm/compare/v4.2.6...v4.2.7
[4.2.6]: https://github.com/Frodo45127/rpfm/compare/v4.2.5...v4.2.6
[4.2.5]: https://github.com/Frodo45127/rpfm/compare/v4.2.4...v4.2.5
[4.2.4]: https://github.com/Frodo45127/rpfm/compare/v4.2.3...v4.2.4
[4.2.3]: https://github.com/Frodo45127/rpfm/compare/v4.2.2...v4.2.3
[4.2.2]: https://github.com/Frodo45127/rpfm/compare/v4.2.1...v4.2.2
[4.2.1]: https://github.com/Frodo45127/rpfm/compare/v4.2.0...v4.2.1
[4.2.0]: https://github.com/Frodo45127/rpfm/compare/v4.1.3...v4.2.0
[4.1.3]: https://github.com/Frodo45127/rpfm/compare/v4.1.2...v4.1.3
[4.1.2]: https://github.com/Frodo45127/rpfm/compare/v4.1.1...v4.1.2
[4.1.1]: https://github.com/Frodo45127/rpfm/compare/v4.1.0...v4.1.1
[4.1.0]: https://github.com/Frodo45127/rpfm/compare/v4.0.22...v4.1.0
[4.0.22]: https://github.com/Frodo45127/rpfm/compare/v4.0.21...v4.0.22
[4.0.21]: https://github.com/Frodo45127/rpfm/compare/v4.0.20...v4.0.21
[4.0.20]: https://github.com/Frodo45127/rpfm/compare/v4.0.19...v4.0.20
[4.0.19]: https://github.com/Frodo45127/rpfm/compare/v4.0.18...v4.0.19
[4.0.18]: https://github.com/Frodo45127/rpfm/compare/v4.0.17...v4.0.18
[4.0.17]: https://github.com/Frodo45127/rpfm/compare/v4.0.16...v4.0.17
[4.0.16]: https://github.com/Frodo45127/rpfm/compare/v4.0.15...v4.0.16
[4.0.15]: https://github.com/Frodo45127/rpfm/compare/v4.0.14...v4.0.15
[4.0.14]: https://github.com/Frodo45127/rpfm/compare/v4.0.13...v4.0.14
[4.0.13]: https://github.com/Frodo45127/rpfm/compare/v4.0.12...v4.0.13
[4.0.12]: https://github.com/Frodo45127/rpfm/compare/v4.0.11...v4.0.12
[4.0.11]: https://github.com/Frodo45127/rpfm/compare/v4.0.10...v4.0.11
[4.0.10]: https://github.com/Frodo45127/rpfm/compare/v4.0.9...v4.0.10
[4.0.9]: https://github.com/Frodo45127/rpfm/compare/v4.0.8...v4.0.9
[4.0.8]: https://github.com/Frodo45127/rpfm/compare/v4.0.7...v4.0.8
[4.0.7]: https://github.com/Frodo45127/rpfm/compare/v4.0.6...v4.0.7
[4.0.6]: https://github.com/Frodo45127/rpfm/compare/v4.0.5...v4.0.6
[4.0.5]: https://github.com/Frodo45127/rpfm/compare/v4.0.4...v4.0.5
[4.0.4]: https://github.com/Frodo45127/rpfm/compare/v4.0.3...v4.0.4
[4.0.3]: https://github.com/Frodo45127/rpfm/compare/v4.0.2...v4.0.3
[4.0.2]: https://github.com/Frodo45127/rpfm/compare/v4.0.1...v4.0.2
[4.0.1]: https://github.com/Frodo45127/rpfm/compare/v4.0.0...v4.0.1
[4.0.0]: https://github.com/Frodo45127/rpfm/compare/v3.0.16...v4.0.0
[3.0.16]: https://github.com/Frodo45127/rpfm/compare/v3.0.15...v3.0.16
[3.0.15]: https://github.com/Frodo45127/rpfm/compare/v3.0.14...v3.0.15
[3.0.14]: https://github.com/Frodo45127/rpfm/compare/v3.0.13...v3.0.14
[3.0.13]: https://github.com/Frodo45127/rpfm/compare/v3.0.12...v3.0.13
[3.0.12]: https://github.com/Frodo45127/rpfm/compare/v3.0.11...v3.0.12
[3.0.11]: https://github.com/Frodo45127/rpfm/compare/v3.0.10...v3.0.11
[3.0.10]: https://github.com/Frodo45127/rpfm/compare/v3.0.9...v3.0.10
[3.0.9]: https://github.com/Frodo45127/rpfm/compare/v3.0.8...v3.0.9
[3.0.8]: https://github.com/Frodo45127/rpfm/compare/v3.0.7...v3.0.8
[3.0.7]: https://github.com/Frodo45127/rpfm/compare/v3.0.6...v3.0.7
[3.0.6]: https://github.com/Frodo45127/rpfm/compare/v3.0.5...v3.0.6
[3.0.5]: https://github.com/Frodo45127/rpfm/compare/v3.0.4...v3.0.5
[3.0.4]: https://github.com/Frodo45127/rpfm/compare/v3.0.3...v3.0.4
[3.0.3]: https://github.com/Frodo45127/rpfm/compare/v3.0.2...v3.0.3
[3.0.2]: https://github.com/Frodo45127/rpfm/compare/v3.0.1...v3.0.2
[3.0.1]: https://github.com/Frodo45127/rpfm/compare/v3.0.0...v3.0.1
[3.0.0]: https://github.com/Frodo45127/rpfm/compare/v2.6.6...v3.0.0
[2.6.6]: https://github.com/Frodo45127/rpfm/compare/v2.6.5...v2.6.6
[2.6.5]: https://github.com/Frodo45127/rpfm/compare/v2.6.4...v2.6.5
[2.6.4]: https://github.com/Frodo45127/rpfm/compare/v2.6.3...v2.6.4
[2.6.3]: https://github.com/Frodo45127/rpfm/compare/v2.6.2...v2.6.3
[2.6.2]: https://github.com/Frodo45127/rpfm/compare/v2.6.1...v2.6.2
[2.6.1]: https://github.com/Frodo45127/rpfm/compare/v2.6.0...v2.6.1
[2.6.0]: https://github.com/Frodo45127/rpfm/compare/v2.5.4...v2.6.0
[2.5.4]: https://github.com/Frodo45127/rpfm/compare/v2.5.3...v2.5.4
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
