# What is MyMod?

A **MyMod** is RPFM's idea of a *mod project* — a Pack that lives in a structured folder on disk, with optional Lua / Sublime / VSCode / Git scaffolding around it, and a few extra menu actions you don't get with a plain Pack.

You can absolutely mod without MyMod. But for any non-trivial project, MyMod gives you:

- A predictable folder layout per game (`<MyMod base>/<game key>/<mod name>/`).
- A spot to keep raw asset files (PSDs, source XMLs, scripts in their working form) alongside the Pack they end up in.
- Optional **Sublime / VSCode** project scaffolding for Lua mods (via the [tw_autogen](https://github.com/Frodo45127/tw_autogen) Lua API definitions).
- Optional **Git** repository initialised inside the MyMod folder, with a configurable `.gitignore`.
- Per-MyMod **Import** (folder → Pack) and **Export** (Pack → folder) actions, so you can work on the mod's contents as files on disk and then bundle them back up.
- Per-MyMod **Delete** that removes the whole folder via the system trash.
- Open and Install/Uninstall through dedicated menu entries.

## The folder layout

When you create a MyMod, RPFM creates the folder and any opt-in scaffolding you ticked in the new-MyMod dialog:

```
<MyMod base>/<game key>/
├── <mod name>.pack                 the Pack itself (saved when you save the MyMod)
└── <mod name>/
    ├── .git/                       only if Git support was enabled
    ├── .gitignore                  only if Git support was enabled
    ├── .vscode/extensions.json     only if VSCode support was enabled
    └── <mod name>.sublime-project  only if Sublime support was enabled
```

The Pack itself also gets a per-Pack **"Files Ignored on Import"** setting populated from the dialog, so the Sublime/VSCode/Git scaffolding files are skipped when you later **Import** the folder back into the Pack.

The `<MyMod base>` is configured once in **PackFile → Settings → Paths → Extra Paths → MyMod base path** ([First-time configuration](../intro/first-time-config.md)). The game-key folders are created on demand the first time you save a MyMod for that game.

## When to use MyMod (and when not to)

**Use MyMod when:**
- You're starting a new mod and want a clean workspace.
- You want one-click Lua / VSCode / Sublime / Git scaffolding.
- The mod has source assets you want to keep alongside the Pack and round-trip via Import/Export.

**Skip MyMod when:**
- You're making one-off edits to an existing Pack you already have on disk.
- You're inspecting / debugging someone else's Pack.

## Active MyMods are per-Pack

A Pack is in **MyMod mode** when its on-disk path lives inside the MyMod base folder under one of the game-key subfolders. The mode is tracked per-Pack on the server: open a normal Pack and a MyMod side by side, and the MyMod-only context-menu actions appear only when you right-click the MyMod's root.

The **MyMod** menu in the menu bar is mostly about creating new MyMods, opening existing ones, and bulk-running Import/Export across every open MyMod. The per-Pack actions (single Import, single Export, Delete, Open MyMod Folder) live in the Pack root's right-click menu.

## Next

- [Creating and managing a MyMod](./managing.md)
- [Import / Export](./import-export.md)
