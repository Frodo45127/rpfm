# Import / Export

MyMod's Import and Export operations move content between a `.pack` file and a folder of files on disk. They're the bridge between "Pack as opaque archive" and "mod content as files I can keep in version control or run scripts against."

Both operations are **per-Pack** and only available when the Pack is in MyMod mode. There are two ways to invoke each:

- **Right-click the Pack root** in the tree and pick **Import** or **Export** — affects just that one MyMod.
- **MyMod → Import All Open MyMods** / **MyMod → Export All Open MyMods** — runs the same operation across every Pack currently open in MyMod mode.

There is no single-Pack equivalent in the menu bar — only the per-Pack right-click and the bulk menu-bar entries.

## Export — Pack → folder

**Export** writes every file inside the MyMod's Pack out to disk under the MyMod folder, preserving the in-Pack tree as on-disk subfolders.

After export you'll have something like:

```
<MyMod base>/<game>/<mod>/
├── <mod>.pack                  the Pack itself (unchanged)
├── db/
│   ├── units_tables/my_units
│   └── ...
├── text/
│   └── my_mod.loc
├── script/
│   └── ...
└── ...
```

DB tables export as TSV by default, and will be parsed from TSV on import automatically.

### Why export?

- **Version control.** Track files individually in git rather than as a single binary blob.
- **External tooling.** Run a script that processes a folder of files (validators, formatters, generators).
- **Bulk renames or refactors** that are easier to do with shell tools than with RPFM's UI.
- **Diffing two Packs** — export both and `diff -r`.

## Import — folder → Pack

**Import** is the reverse: scan the MyMod folder, collect every file under it (excluding any path listed in the Pack's **Files Ignored on Import** setting), and add them to the Pack. Files at the same in-Pack path are replaced with the on-disk version.

**Import is additive.** Files that exist in the Pack but no longer exist on disk are *not* deleted. If you've removed a file from the on-disk folder and want it gone from the Pack too, delete it from the Pack manually (right-click → Delete).

### The ignore list

The **Files Ignored on Import** list is the per-Pack setting you (optionally) populated when you created the MyMod. It lives in the Pack itself and is editable later via **Open PackFile Settings** ([Pack settings & notes](../packs/pack-settings.md)).

The new-MyMod dialog also auto-appends entries for any Sublime / VSCode / Git scaffolding you opted in to, so you don't accidentally import `.git/`, `.vscode/`, the Sublime project file, or `.luarc.json` back into the Pack.

## Round-tripping

Export → edit on disk → Import is the standard "I want to script changes to this mod" workflow. As long as the on-disk layout matches the in-Pack layout, RPFM treats it as a faithful round-trip.

> Importing a folder that was exported from a different RPFM install (e.g. via git from a teammate) works the same. The MyMod folder layout is the contract — and the per-Pack ignore list lives inside the Pack itself, so it travels with the `.pack`.

## Caveats

- **Import is additive, not destructive.** Removing a file from the on-disk folder won't remove it from the Pack on the next Import — delete it from the Pack manually.
- **Binary files** (DDS, RigidModel, AnimPack, BMD, …) are exported as-is. Edit them in the appropriate external tool, save them back to the same path, and re-import.
