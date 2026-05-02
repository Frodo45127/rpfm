# The Pack tree

The Pack tree is the left-hand panel listing the contents of every open Pack. Files are grouped by their in-Pack path (DB tables under `db/`, locs under `text/`, etc.). The tree is the entry point to almost every per-file action in RPFM.

<!-- IMAGE: Pack tree with one Pack expanded, the context menu open over a DB table file, and the filter bar visible at the top. -->

## Navigating

- **Double-click a file** to open its editor in a new tab.
- **Single-click** to select, and to open its editor in a preview tab. Multi-select with `Shift` (range) or `Ctrl` (additive).
- **Right-click** to open the context menu (covered below).
- **Filter** with the search box at the bottom of the tree. Matches against file paths; the tree collapses to show only matching entries.

Modified or newly-added items are marked with a **thin vertical line on the right side** of their row. Two states are drawn:

- **Added** — new file/folder since the Pack was opened (or since the last save).
- **Modified** — file with unsaved changes in the Pack (or a folder containing modified children). Modified takes priority over Added when both apply.

Both colours are configurable in **Preferences → Appearance** (`Added` and `Modified` colours, with separate light/dark theme entries). Folders inherit the most "active" state of their children. MyMod Pack roots also get an italic "MyMod" label drawn on the right side of the row.

## Context menu

Right-clicking a file or folder gives you the actions below. Most also have keyboard shortcuts you can rebind in **Preferences → Shortcuts**.

The actions are grouped under three submenus (**Add…**, **Create…**, **Open…**) plus a flat list of file operations and Pack-level actions.

### Add…

- **Add File** — pick one or several files from disk and add them to the Pack at the selected location.
- **Add Folder** — recursively add every file under a folder. The directory structure is preserved.

### Create…

- **New Folder** — create an empty folder inside the Pack.
- **New AnimPack / DB / Loc / Portrait Settings / Text** — create an empty file of the chosen type at the selected location.
- **New Quick File** — create a file with its type autodetected, based on the folder you have selected.

### Open…

- **Open with Decoder** — open the file in the [DB Decoder](../editors/decoder.md). Used only for decoding tables.
- **Open Dependency Manager** — opens the in-Pack dependency manager. See [Dependencies](./dependencies.md).
- **Open Containing Folder** — opens the folder containing your pack in your file manager.
- **Open with External Program** — open the file in the external program you have configured in your OS for its type. Edits are reloaded back into the Pack when you save in the external editor.
- **Open PackFile Settings** — opens [Pack settings](./pack-settings.md) for the Pack the selection belongs to.
- **Open Notes** — opens the Notes view for the Pack.

### File operations

- **Rename/Move** — rename the file or folder, or move it to a different in-Pack path. Renaming a folder rewrites every contained path; references in DB tables are *not* updated automatically.
- **Delete** — remove the selection from the Pack.
- **Extract** — write the selected files out to disk, preserving their tree structure.
- **Copy Path** — copy the in-Pack path to the system clipboard.
- **Copy / Cut / Paste** — clipboard operations within the Pack tree (or back into the same Pack at a new location).
- **Duplicate** — create a copy of the selected file inside the Pack.
- **Copy To Pack ▸ \<other Pack\>** — copy the selection into another open Pack. Useful when refactoring or splitting mods.
- **Merge Tables** — only enabled with multiple DB tables of the same definition (or multiple Loc files) selected. Combines them into one and removes the originals. Useful for collapsing many small tables into a single one.
- **Update Tables** — update the tables in the pack to their latest version supported by the game.
- **Generate Missing Loc Data** — generates Loc entries for fields in your pack that are missing them.

### Pack-root actions

When you right-click the Pack root itself, additional Pack-level actions appear: **Save Pack**, **Save Pack As…**, **Save Pack For Release…**, **Close Pack**, **Install / Uninstall**, the **Change PackFile Type** submenu (Boot, Release, Patch, Mod, Movie, plus header flags), the **Compression Format** submenu (None, Lzma1, Lz4, Zstd), the Special Stuff actions for the active game (Optimize PackFile, Rescue PackFile, Build Starpos, Patch Siege AI, Live Export, Pack Map, Update Anim IDs), and MyMod actions when the Pack is a MyMod.

## Multi-pack operations

Some context-menu actions know about every open Pack:

- **Copy To Pack ▸** lists every other open Pack.
- **Cut**+**Paste** moves files within a Pack at a new location.

The tree itself only supports drag-drop *within* a single Pack. To move files between Packs, use **Copy To Pack** or **Cut**+**Paste**.

## Tips

- The filter bar accepts substrings and regex. Use it to surface every loc, every animation, every file under `text/db/`, etc.
- Use **Expand All / Collapse All** at the bottom of the context menu to quickly fold or unfold the tree.
