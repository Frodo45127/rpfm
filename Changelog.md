# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project doesn't adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
