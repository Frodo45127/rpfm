# Keyboard shortcuts

RPFM lets you rebind every shortcut from **PackFile → Settings → Shortcuts** (the Preferences dialog has a **Shortcuts** button at the bottom). The dialog itself is the standard KDE `KShortcutsDialog`, which groups every action by collection and flags conflicts within the same scope.

The list below shows the **defaults** as defined in the source. Most shortcuts are configurable per scope, so a binding may legitimately fire in one widget (e.g. the table editor) and be unbound in another.

> **The command palette is the universal fallback.** Press `Ctrl+P` (file palette) or `Ctrl+Shift+P` (command palette) and type the name of what you want. If the action exists, you can launch it from there.

## Pack Menu

| Action                  | Default        |
|-------------------------|----------------|
| New Pack                | `Ctrl+N`       |
| Open Packs              | `Ctrl+O`       |
| Open & Merge Packs      | `Ctrl+Shift+O` |
| Save All Packs          | `Ctrl+S`       |
| Save Current Pack       | `Ctrl+Shift+S` |
| Install Pack            | `Ctrl+Shift+I` |
| Uninstall Pack          | `Ctrl+Shift+U` |
| Load All CA Packs       | `Ctrl+G`       |
| Settings                | `Ctrl+,`       |
| Quit                    | (unbound)      |

`Save All Packs` (`Ctrl+S`) saves every open Pack; `Save Current Pack` (`Ctrl+Shift+S`) saves only the Pack the focused file/tab belongs to. Save Pack As is exposed via the Pack root context menu and the file-tab context menu rather than as a global shortcut.

## File tabs

| Action                  | Default          |
|-------------------------|------------------|
| Close Tab               | `Ctrl+W`         |
| Previous Tab            | `Ctrl+Shift+Tab` |
| Next Tab                | `Ctrl+Tab`       |
| Toggle Quick Notes      | (unbound)        |
| Import From Dependencies| (unbound)        |

## View menu

| Action                  | Default        |
|-------------------------|----------------|
| Toggle Global Search    | `Ctrl+Shift+F` |
| Toggle Pack Contents    | (unbound)      |
| Toggle Diagnostics      | (unbound)      |
| Toggle Dependencies     | (unbound)      |
| Toggle References       | (unbound)      |

## Command palette

| Action                  | Default        |
|-------------------------|----------------|
| Open file palette       | `Ctrl+P`       |
| Open command palette    | `Ctrl+Shift+P` |

## Pack tree (context-scoped)

| Action                       | Default              |
|------------------------------|----------------------|
| Add File                     | `Ctrl+A`             |
| Add Folder                   | `Ctrl+Shift+A`       |
| Add From Pack                | `Ctrl+Alt+A`         |
| New Folder                   | `Ctrl+F`             |
| New DB                       | `Ctrl+D`             |
| New Loc                      | `Ctrl+L`             |
| New Text                     | `Ctrl+T`             |
| New Quick File               | `Ctrl+Q`             |
| New AnimPack / Portrait Settings | (unbound)        |
| Merge Files                  | `Ctrl+M`             |
| Delete                       | `Del`                |
| Extract                      | `Ctrl+E`             |
| Rename / Move                | `Ctrl+R` or `F2`     |
| Copy                         | `Ctrl+C`             |
| Cut                          | `Ctrl+X`             |
| Paste                        | `Ctrl+V`             |
| Duplicate                    | `Ctrl+Shift+D`       |
| Open with Decoder            | `Ctrl+J`             |
| Open with External Program   | `Ctrl+K`             |
| Open Pack Notes              | `Ctrl+Y`             |
| Expand All                   | `Ctrl++`             |
| Collapse All                 | `Ctrl+-`             |

## Table editor (DB / Loc)

| Action                          | Default         |
|---------------------------------|-----------------|
| Add Row                         | `Ctrl+Shift+A`  |
| Insert Row                      | `Ctrl+I`        |
| Delete Row                      | `Ctrl+Del`      |
| Delete Filtered-Out Rows        | `Ctrl+Shift+Del`|
| Clone & Insert Row              | `Ctrl+D`        |
| Clone & Append Row              | `Ctrl+Shift+D`  |
| Copy                            | `Ctrl+C`        |
| Copy as LUA Table               | `Ctrl+Shift+C`  |
| Paste                           | `Ctrl+V`        |
| Paste as New Row                | `Ctrl+Shift+V`  |
| Rewrite Selection               | `Ctrl+Y`        |
| Invert Selection                | `Ctrl+-`        |
| Search                          | `Ctrl+F`        |
| Undo / Redo                     | `Ctrl+Z` / `Ctrl+Shift+Z` |
| Smart Delete                    | `Del`           |
| Find References / Go To Definition / Go To File / Patch Columns / Generate IDs / Import TSV / Export TSV / etc. | (unbound) |

## Decoder

| Action            | Default       |
|-------------------|---------------|
| Move Field Up     | `Ctrl+Up`     |
| Move Field Down   | `Ctrl+Down`   |
| Move Field Left   | `Ctrl+Left`   |
| Move Field Right  | `Ctrl+Right`  |
| Delete Field      | `Ctrl+Del`    |
| Delete Definition | `Ctrl+Del`    |
| Load Definition   | `Ctrl+L`      |

## Text editor

The text editor is a **KTextEditor** widget and honours your KDE keyboard configuration. RPFM removes its built-in file-save and file-save-as actions and re-binds save into the Pack via `Ctrl+S`. Find / replace use KTextEditor's standard `Ctrl+F` / `Ctrl+R`. Everything else is KTextEditor-standard.

## Customising

The Shortcuts dialog (PackFile → Settings → Shortcuts) shows every action grouped by collection (Pack Menu, View Menu, Pack Tree Context Menu, Table Editor, Decoder, etc.). Click a binding to edit it; conflicts within the same scope are flagged.

Bindings are persisted via KDE's standard `KActionCollection::writeSettings`/`readSettings`, not in a single RPFM-owned file. On Linux that typically lands in `~/.config/<orgname>rc`. To reset, open the Shortcuts dialog and use its built-in **Defaults** action.
