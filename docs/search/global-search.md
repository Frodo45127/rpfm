# Global Search

The Global Search panel finds — and optionally replaces — text across every supported file type in the open Packs (and, optionally, in vanilla and parent mods). It's the closest thing RPFM has to a project-wide grep.

<!-- IMAGE: Global Search panel docked at the bottom, with a search pattern entered, source toggles visible, and a results tree showing matches grouped by file. -->

Toggle the panel from **View → Toggle Global Search Window** (default shortcut `Ctrl+Shift+F`).

## Pattern & options

- **Pattern.** What to match. By default it's a literal substring search.
- **Use Regex.** Treat the pattern as a regular expression instead.
- **Case sensitive.** Off by default.

There is no "match whole word" toggle — wrap the pattern in word boundaries yourself in regex mode (`\bword\b`) if you need that behaviour.

## Sources

A search can target several sources independently:

- **Per-Pack checkboxes.** One checkbox per open Pack appears under the Sources group; tick the ones you want included.
- **Game Files.** Vanilla files for the active game (requires the [dependencies cache](../packs/dependencies.md)).
- **Parent Files.** Parent mods declared by the active Pack (see [Dependencies](../packs/dependencies.md)).
- **Assembly Kit Files.** AK-only tables and locs (requires the AK install path to be configured for the active game).

You can mix any combination. The result tree groups matches per source.

## File types covered

The search has a per-file-type checkbox for every editable type RPFM knows about. There's also a **Search on All** master toggle and a **Search on All Common** toggle (DB / Loc / Text — the day-to-day editable types).

For DB tables and Loc files the matches are cell-level with row context; for text-style files matches are line-level; for the structured types (Atlas, Portrait Settings, etc.) matches are field-level inside the file's structure.

## Working with results

Each match in the results tree is clickable: clicking a DB cell opens the table and selects the cell, clicking a Loc match opens the right loc file at the right key, and so on.

Multi-select results to scope a follow-up replace.

## Replace

Two modes:

- **Replace match** — replace the currently-selected match(es) with the replacement text.
- **Replace all matches** — replace every match in the result set in one shot.

Replace operations only run on Packs (Game / Parent / AK sources are read-only and skipped automatically).

> Replace is destructive. Make sure you've saved (or have a recent autosave) before doing a project-wide replace, especially with regex.
