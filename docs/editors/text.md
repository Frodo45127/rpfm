# Text & scripts

Anything text-shaped opens in the text editor. RPFM uses **KTextEditor** under the hood, which means proper syntax highlighting, line numbers, find/replace, multi-cursor and the rest of the modern code-editor toolkit — for free, on every supported language.

<!-- IMAGE: Text editor open on a Lua script with syntax highlighting and the find bar visible. -->

## What counts as "text"

RPFM auto-detects text-shaped files by extension. The editor handles, among others:

- **Scripts** — Lua, Python, batch files, shell.
- **Markup** — XML, HTML, JSON, YAML, TOML, INI.
- **Shaders** — HLSL, GLSL, CG, FX.
- **CA-specific** — `.battle_script`, `.twui`, `.twui.xml`, `.kfa`, `.kfc`, `.kfp`, `.tweak`, `.environment`.
- **Plain** — `.txt`, `.md`, `.csv` (when not opened as a table).

For the full list, see the [`rpfm_lib::files::text` module](../../api/rpfm_lib/files/text/index.html).

## Features

The KTextEditor backend gives you:

- **Syntax highlighting** for every detected language.
- **Find / replace** (`Ctrl+F` / `Ctrl+R`) with regex support.
- **Multi-cursor / column selection** (`Ctrl+Alt+click`).
- **Bookmarks** and quick-jump.
- **Code folding** for languages that support it.
- **Indent / unindent** (`Tab` / `Shift+Tab`).
- **Open / save** integration with the Pack — your edits go into the in-memory Pack on save.

## External editor

If you'd rather use VS Code, Neovim or anything else, **Open with External Program** from the Pack tree's right-click menu extracts the file to a temp folder and opens it with your OS's default app for that extension; saving it externally pushes the change back into the Pack. There's no per-extension override inside RPFM — the OS picks the app.
