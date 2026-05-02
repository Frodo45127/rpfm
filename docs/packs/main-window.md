# The main window

A quick orientation tour. Once you know where everything lives, the rest of the manual will make sense as it points at this or that panel.

<!-- IMAGE: Main window with annotations: menu bar (1), tab bar (2), Pack tree (3), editor area (4), bottom panel docks (5), status bar (6). -->

## The pieces

1. **Menu bar.** Top of the window. Houses every global action — see below.
2. **Tab bar.** Sits under the menu bar. One tab per open file.
3. **Pack tree.** Left dock, top half. Shows the files inside the open Packs. Right-click anything in the tree for context actions; double-click a file to open its editor in a new tab.
4. **Dependencies.** Left dock, under the Pack tree. Shows the contents of the active game's dependencies cache (vanilla + parent mod Packs), so you can browse them alongside your own files.
5. **Editor area.** The big central region. Whatever tab is active here drives most of what you can do.
6. **Side and bottom panels.**
    - **Diagnostics** — bottom dock, visible by default. Lint-style warnings for the open Packs.
    - **Global Search** — right dock, hidden by default. Toggle from **View** or with its shortcut.
    - **References** — bottom dock, hidden by default. Populated when you ask "what points at this row?" from an editor.
    - **Quick Notes** — not a dock. It's a side panel inside each file editor, toggled per-tab.
7. **Status bar.** Bottom strip. Shows short-lived status messages from the last operation.

## The welcome page

When no Pack is open, the editor area is replaced by a **welcome page** with quick links to recent Packs, the manual, the GitHub project, the Patreon (wink wink), and some other useful stuff.

<!-- IMAGE: Welcome page with the recent Packs list and the tip footer visible. -->

## The command palette

Press `Ctrl+P` (or `Ctrl+Shift+P` for the action variant) anywhere in the app to open the **command palette**. It's a fuzzy finder for:

- Open files (jump to any file inside the Pack by typing part of its path).
- Available commands (run any menu action by typing its name).

For most "where is the menu item that does X?" questions, the command palette is the answer.

## The menu bar

| Menu | What it covers |
|------|----------------|
| **Pack** | Pack lifecycle: new, open, save (single Pack or all open ones), save for release, close, recent files, open from content / autosave / data / secondary, session picker, Pack settings, quit. |
| **MyMod** | The MyMod project workflow: open the MyMod folder, create a new MyMod, import / export all open MyMods in one go, and a per-game submenu listing the existing MyMods for that game so you can open them directly. Deleting a MyMod now lives in the Pack tree's right-click menu (on a MyMod Pack's root). See [What is MyMod?](../mymod/overview.md). |
| **View** | Toggle the bottom and side docks (Diagnostics, Global Search, References, Quick Notes, Dependencies). |
| **Game Selected** | Switch the active Total War game, launch it, open its data / Assembly Kit folders, open RPFM's config folder, and regenerate the dependencies cache.
| **Tools** | Integrated tools that span multiple files: **Translator**, **Faction Painter**, **Unit Editor**. |
| **About** | Manual, repo, Patreon, Discord, version info, check for updates. |
| **Debug** | Hidden by default. Enable it from **Preferences → Debug** to expose lower-level commands useful for development. |

## Connecting the dots

If you want a focused walk-through of a particular task, the rest of this section covers Pack-level workflows:

- [Opening, creating and saving Packs](./pack-operations.md)
- [The Pack tree](./pack-tree.md)
- [Pack settings & notes](./pack-settings.md)
- [Dependencies](./dependencies.md)

For the editors themselves (DB, Loc, Text, etc.), see the [Editors](../editors/overview.md) section. For everything Pack-relationship-related (search, diagnostics, references), see [Search & analysis](../search/global-search.md).
