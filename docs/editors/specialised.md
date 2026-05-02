# Specialised editors

Several less-common file types have dedicated views that didn't warrant their own chapter. Grouped here for completeness.

## UnitVariant

Variant mesh definitions for units — which RigidModel goes with which faction colour scheme, kit, etc. The view is a proper structured editor: a categories list on the left, a variants list in the middle for the selected category, and a per-variant form on the right (mesh file, texture folder, an unknown numeric value). Add / clone / delete on both lists from their right-click menus.

A separate read-only **JSON debug view** for the same file type also exists behind a settings, in case you want to edit the file more freely.

<!-- IMAGE: UnitVariant editor with a categories list, variants list and the per-variant form. -->

## Atlas

Sprite-sheet layout files. Each row defines a region (UV rectangle + name) within an associated texture. Atlas files open in the regular [DB editor](./db.md) — full grid editing, TSV round-trip, the whole DB editor toolset.

## ESF

`.esf` is CA's binary format for save games and startpos files. Massive, deeply nested, and slow to parse. RPFM exposes it as a tree of nested nodes with typed leaves (ints, floats, strings, arrays).

> **Disabled by default.** The ESF editor lives behind the **Enable ESF Editor** toggle in **PackFile → Settings → Debug**. Turn it on first; otherwise opening an `.esf` falls back to the JSON debug view.

- **Browse** the full structure as a tree.
- **Edit** leaf values in place via a side detail panel.
- **Filter** within the loaded ESF (substring + regex toggle, plus an "auto-expand matches" option).

Writing is supported but slow on big files (a multi-hundred-MB save can take a few seconds). Most ESF editing is for **startpos** files in a campaign mod context.

<!-- IMAGE: ESF editor showing the tree of nodes on the left and the leaf editor on the right. -->

## BMD

Battle Map Definition files — the binary scene description for battle maps (terrain, props, lighting). The current view is a **JSON text editor**: the file decodes via the lib, gets serialised to pretty JSON, and saves parse the JSON back. Editable but unstructured.

## Group formations

Formation definitions for unit groups (e.g. infantry block, cavalry charge wedge). UI is a **JSON debug view** — the lib supports full read/write for the per-game variants (Warhammer 3 / Three Kingdoms / Troy / Rome 2 / Shogun 2 each have their own decode path), but the editor is the generic JSON editor.

## Animation file formats summary

For convenience, here's where each animation-shaped format lives in the manual:

| Format                    | Editor chapter                         |
|---------------------------|----------------------------------------|
| AnimPack                  | [AnimPack](./animpack.md)              |
| AnimsTable                | [Animations](./animations.md)          |
| AnimFragmentBattle        | [Animations](./animations.md)          |
| MatchedCombat             | [Animations](./animations.md)          |
| `.anim` raw stream        | No UI viewer today                     |
