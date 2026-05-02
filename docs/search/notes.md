# Quick Notes

The Quick Notes panel surfaces the **active file's notes** inline at the side of the editor, so you don't need to open the dedicated notes view every time you want to read or jot something down.

Toggle it from the file tab's right-click menu → **Toggle Quick Notes**. There is no main-menu **View** entry for it, and no global shortcut by default — every editor tab gets its own panel that you turn on per-tab.

<!-- IMAGE: Quick Notes panel docked on the right side of an editor tab showing a list of notes attached to the file. -->

## What notes actually are

Notes in RPFM are a **structured list**, not a single free-form text area. Each note has:

- A short message (the body).
- An optional URL (for linking out to a wiki page, issue tracker, etc.).
- An implicit attachment: either to the Pack as a whole, or to a specific in-Pack path.

The Quick Notes panel for a file shows every note attached to that file's path, plus a **+** to add a new one and right-click **Edit** / **Delete** on each entry.

## What it shows

When the focused editor changes (e.g. you switch tabs), each tab's Quick Notes panel is independent and shows the notes for *that* tab's file. There's no automatic fall-back to "Pack notes" — Pack notes are accessed through the dedicated Notes view (see below).

For files opened from the Dependencies panel (vanilla / parent / AK files) Quick Notes is read-only and effectively empty: notes only exist for files that live in an open Pack.

## Difference from "Open Notes"

- **Open Notes** (Pack tree context menu → **Open ▸ Open Notes**) opens a dedicated tab containing the same structured list view. Use this for browsing every note attached to a Pack, or for working with notes when you don't have the relevant file open.
- **Quick Notes panel** is the same view embedded inside the active editor tab so you can read / jot without leaving the file.

## A reminder about Save for Release

**Save for Release does not strip notes.** It runs the optimiser dialog before saving but leaves the Pack's notes file (`notes.rpfm_reserved`) intact for `Mod` and `Movie` Packs. Notes are dropped only if you change the Pack type to `Boot` / `Release` / `Patch` before saving — at which point the notes section, the dependency manager and the per-Pack settings are all skipped.

Notes are intended for your own bookkeeping; if you don't want them shipping with the Pack, delete them or change the Pack type before release.
