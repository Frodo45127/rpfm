# Diagnostics panel

Diagnostics is RPFM's static analyser for Packs. It runs a battery of checks across the open Packs and surfaces structured warnings and errors — invalid references, missing locs, broken portrait variants, datacore mistakes, and dozens of other classes of mod bug.

Toggle the panel from **View → Toggle Diagnostics Window**.

<!-- IMAGE: Diagnostics panel showing a results table with severity icons (info/warning/error), the file path, the diagnostic type, and a short description. -->

## What it checks

Diagnostics is grouped internally by check category (one variant of `DiagnosticType` per category):

| Category               | Sample checks |
|------------------------|----------------|
| **Tables**             | Outdated table version, invalid foreign-key references, empty rows, empty keys, keys with invalid characters (jumplines, tabs, spaces, or trailing whitespace), duplicated rows, datacore vs. master mismatches, tables on the modding deny-list, table-name issues (trailing number, contains space), required fields left empty, altered / out-of-spec tables, dangling file-path field references, invalid escape sequences in text. |
| **Pack-level**         | Invalid Pack name, invalid file names, missing loc data for referenced keys, ITM (identical-to-master) files, files overwriting vanilla unintentionally, duplicate files. |
| **Portrait Settings**  | Art sets / variants not declared in the corresponding tables, missing texture references, datacored portrait_settings files. |
| **AnimFragmentBattle** | Missing locomotion graphs, missing animation files, missing sound files, missing metadata file. |
| **Text**               | Invalid loc-key references in scripts. |
| **Dependencies**       | Pack declares a parent that doesn't exist on disk. |
| **Config**             | Missing or outdated dependency cache, dependency cache failed to load, wrong game path. |

Every diagnostic carries a severity (`Info`, `Warning`, `Error`), a category, the file path it applies to, the row/cell/field it relates to, and a short description.

## Running diagnostics

- **Run all** — checks every file in every open Pack.
- **Run on open files** — limited scope; useful in big Packs.
- **Auto-run** — two togglable triggers in **PackFile → Settings → Diagnostics**:
  - *Trigger diagnostics on Pack open*
  - *Trigger diagnostics on table edit* (only fires while the diagnostics dock is visible)

There are no "on file add" or "on Pack save" auto-run triggers.

## Triaging results

**Click a column header** to sort the results by that column (severity, file, type, etc.), so you can group everything of one kind together or push all the errors to the top.

Double-click a diagnostic to jump to the offending file / row / cell. Right-click for the ignore actions:

- **Ignore Parent Folder** / **Ignore Field for Parent Folder** — skip every diagnostic on every file in the parent folder, optionally narrowed to a specific field.
- **Ignore File** / **Ignore Field for File** — skip every diagnostic on this file, optionally narrowed to a specific field.
- **Ignore Diagnostic for Parent Folder** / **Ignore Diagnostic in Field for Parent Folder** — skip just this specific diagnostic type across the parent folder, optionally narrowed to a field.
- **Ignore Diagnostic for File** / **Ignore Diagnostic in Field for File** — same, scoped to one file.
- **Ignore Diagnostic for Pack** — skip this diagnostic type across the whole Pack.

There is no "Fix automatically" or "Copy diagnostic" action today — every fix is manual.

## Where ignores live

Each ignore action appends one line to the Pack's **Files Ignored on Diagnostics Check** setting (the multi-line text field exposed in the Pack Settings tab — see [Pack settings & notes](../packs/pack-settings.md)). The Pack Settings tab also documents the per-file / per-field / per-diagnostic syntax if you want to write entries by hand.

These ignores live inside the Pack itself, survive across RPFM restarts, and travel with the `.pack` to anyone else who opens it.

## Reading severities

- **Error** — something that will cause a problem in-game.
- **Warning** — likely a problem, but might not be a problem in-game.
- **Info** — heads-up; noteworthy but not necessarily wrong.

If you're shipping a mod, aim for zero errors and as few warnings as possible. Info entries are usually fine to leave alone.
