# Creating and managing a MyMod

The MyMod actions are split between the **MyMod** menu in the menu bar (creation, opening, bulk operations) and the Pack root's right-click menu (per-Pack operations on a MyMod).

<!-- IMAGE: MyMod menu open showing "Open MyMod Folder", "New MyMod", "Import All Open MyMods", "Export All Open MyMods", and the per-game submenus listing existing MyMods. -->

## The MyMod menu

In the menu bar the **MyMod** menu has, in order:

- **Open MyMod Folder** — opens the configured `<MyMod base>` in your file manager.
- **New MyMod** — creates a new MyMod (covered below).
- **Import All Open MyMods** — runs **Import** on every Pack currently open in MyMod mode.
- **Export All Open MyMods** — runs **Export** on every Pack currently open in MyMod mode.
- *(separator)*
- **\<Game name\>** submenu — one per supported game, *only visible* if you have at least one MyMod for that game. Each submenu lists the `.pack` files inside `<MyMod base>/<game key>/` as one click-to-open entry per mod.

**New MyMod**, **Import All** and **Export All** are disabled until a MyMod base path is configured in **PackFile → Settings → Paths → Extra Paths**.

## Creating a MyMod

**MyMod → New MyMod** opens the new-MyMod dialog. The fields are:

- **Game** — combobox of supported games (Arena is excluded). Defaults to the currently-selected game.
- **Mod Name** — short identifier. Used as both the folder name and the Pack file name. **No spaces allowed**; the Accept button is disabled if the name contains spaces or is already in use.
- **Lua Support** group:
  - **Create Sublime Text Project** — drops a `<mod_name>.sublime-project` in the folder.
  - **Create VSCode Project** — drops `.vscode/extensions.json` in the folder, recommending the Lua and Code Runner extensions.
  - When either is enabled, RPFM also expects a `.luarc.json` to live there (auto-added to the import-ignore list).
- **Create Git Repository With GitIgnore** group (checkable) — when enabled, RPFM `git init`s the folder and writes a `.gitignore`. You can either type your own `.gitignore` contents in the **GitIgnore Contents** textedit, or tick **Same as files ignored on import** to mirror the import-ignore list.
- **Files Ignored on Import** — paths (one per line, relative to the MyMod folder) that RPFM should skip when later running **Import**. The Sublime/VSCode/Git scaffolding paths are appended automatically.

On confirm, RPFM:

1. Creates `<MyMod base>/<game key>/<name>/` on disk.
2. Initialises `.git`, `.gitignore`, `.vscode/`, `<name>.sublime-project` as opted in.
3. Creates an empty Pack and saves it as `<name>.pack` inside the folder.
4. Stores the import-ignore list in the Pack's settings.
5. Switches the active Game Selected to match the MyMod's game.
6. Opens the Pack in a new tab and marks it as a MyMod on the server.

## Switching MyMods

Use the per-game submenus in **MyMod → \<Game name\> → \<mod name\>** to open an existing MyMod. The previously-open Packs (MyMods or otherwise) are left untouched — opening a MyMod just opens that Pack alongside the others.

## Per-MyMod actions (right-click the Pack root)

When you right-click the root of a Pack that's in MyMod mode, the context menu adds four MyMod-only entries (greyed out / hidden for non-MyMod Packs):

- **Import** — folder → Pack for this MyMod. See [Import / Export](./import-export.md).
- **Export** — Pack → folder for this MyMod. See [Import / Export](./import-export.md).
- **Delete Selected MyMod** — moves the entire MyMod folder to the system trash after a confirmation. Cannot be undone from inside RPFM.
- **Open MyMod Folder** — opens the MyMod's folder in your file manager.

## Installing to the game folder

Install / Uninstall are not MyMod-specific menu items — they're plain Pack actions that show up in the Pack root's right-click menu for any saved Pack:

- **Install** copies `<pack path>` into `<game install>/data/<pack name>` so you can launch and test.
- **Uninstall** removes that copy.

For a "release" install (with the optimiser dialog), use **Pack → Save Pack For Release** before installing — see [Pack operations](../packs/pack-operations.md).
