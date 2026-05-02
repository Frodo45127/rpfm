# Pack settings & notes

Each Pack carries a small bundle of metadata that lives inside the Pack itself. These are spread across a few different views and menu actions in the UI; this page covers them.

<!-- IMAGE: Pack Settings tab open with the diagnostics-ignore field expanded, plus the Pack root context menu showing the Change PackFile Type and Compression Format submenus. -->

## Pack file type, compression, header flags

These live on the Pack root's context menu (right-click the Pack in the tree). They are *not* inside the Pack Settings tab.

### Change PackFile Type

A submenu listing the possible PFH file types: **Boot**, **Release**, **Patch**, **Mod**, **Movie**. Mods almost always want **Mod** (shows up in the launcher) or **Movie** (always-on, hidden from the launcher). The other three are CA-only types and should not be used for player mods.

The same submenu also exposes the header flags as checkable items: **Index Includes Timestamp** (editable), and **Header Is Extended**, **Index Is Encrypted**, **Data Is Encrypted** (read-only — RPFM cannot save encrypted Packs, but it can read them).

### Compression Format

A submenu listing the supported compression formats: **None**, **Lzma1**, **Lz4**, **Zstd**. Which formats actually work depends on the active game. The selected format is applied the next time the Pack is saved.

## Pack Settings tab

**Open Pack Settings** (Pack root context menu → **Open ▸ Open PackFile Settings**) opens the Pack Settings as a tab in the main view. It's a flat scrolling form that holds RPFM-specific behavioural settings for the Pack:

- **PackedFiles to Ignore on Diagnostics Check** — paths (one per line, `#` for comments) that the diagnostics tool should skip. Supports per-folder, per-file, per-field and per-diagnostic ignore patterns. Files in this list still contribute reference data — they just aren't analysed.
- **Files to Ignore when Importing** — paths to skip when importing from a MyMod folder. MyMod-only.
- **Disable Autosaves for this PackFile** — turns off autosaving for just this Pack.
- **Do Not Generate Existing Locs** — when checked, **Generate Missing Locs** skips Loc entries that already exist in vanilla or in parent files.

Click **Apply Settings** at the bottom to save changes back into the Pack.

> The fields shown here are loaded dynamically from the Pack's settings, so the exact list may grow over time as new options are added.

## Notes

Notes in RPFM are *structured* — not a single free-form text area, but a list of short notes, each with a message and an optional URL. Notes can be attached to **The Pack as a whole** — created from the tree's context menu → **Open ▸ Open Notes** with the Pack root selected (or with no file selected).

Notes are persisted inside the Pack, are visible in RPFM, and are ignored by the game.

### Quick Notes panel

The active file editor includes a **Quick Notes** side panel that lists the notes attached to the open file. Toggle it from the file tab's right-click menu → **Toggle Quick Notes**.

## Where the data lives

Pack settings, notes and the dependency manager are stored in a small set of RPFM-reserved files inside the Pack (`settings.rpfm_reserved`, `notes.rpfm_reserved`, `dependencies_manager_v2.rpfm_reserved`). The game ignores these; other tools that don't know about RPFM will see them as normal files. They round-trip cleanly between RPFM versions.

These reserved files are only written for Packs of type **Mod** or **Movie**. Switching the Pack to **Boot**, **Release** or **Patch** before saving therefore strips them.
