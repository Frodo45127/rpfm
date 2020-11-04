# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project doesn't adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[Unreleased]: https://github.com/Frodo45127/rpfm/compare/v2.1.5...HEAD
[2.1.5]: https://github.com/Frodo45127/rpfm/compare/v2.1.4...v2.1.5
[2.1.4]: https://github.com/Frodo45127/rpfm/compare/v2.1.3...v2.1.4
