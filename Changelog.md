# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project doesn't adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- Implemented Autosave.
- Implemented `Are you sure?` dialog when closing the main window.
- Implemented shortcuts to close/move to next/move to prev tab.
- Implemented the `Use Old Column Order (Keys first)` setting for people who prefer pre-2.x column order in tables.
- Implemented `Paste As New Row` for tables.

### Changed
- Changed all fields that accept regex so it turns green or red depending if the regex is valid or not.
- Tweaked TreeView Colours to be more visible on light theme.
- Now the search field gets focused when opening the Global/Table Search pannels.
- Images now are shown in their original size, except if they are bigger than the current window.

### Fixed
- Fixed a CTD when starting RPFM without internet connection.
- Fixed a CTD caused by using Ctrl+Z after pasting a reference cell with the dependency checker enabled.
- Fixed a couple of ordering bugs in the table column order logic.
- Fixed regex coloring on Global Search not working if you used the dark theme.

## [2.1.5] - 2020-08-22
### Added
- Implemented Autoupdater.

## [2.1.4] - 2020-08-15
- For this update and older ones, check the release page.

[Unreleased]: https://github.com/Frodo45127/rpfm/compare/v2.1.5...HEAD
[2.1.5]: https://github.com/Frodo45127/rpfm/compare/v2.1.4...v2.1.5
[2.1.4]: https://github.com/Frodo45127/rpfm/compare/v2.1.3...v2.1.4
