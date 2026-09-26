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

Formation templates the AI (and multiple-selection drag-outs) use to deploy armies. Supported for Shogun 2, Rome 2, Attila, Thrones of Britannia, Troy, Pharaoh, Three Kingdoms and Warhammer 3.

Each formation is a set of **blocks**:

- **Absolute blocks** sit at a fixed position. Every formation needs at least one, as it anchors the rest.
- **Relative blocks** sit at an offset from another block, their parent.
- **Spans** group other blocks, so relative blocks can be positioned from the whole group.

The editor has three columns:

- **Formations**: the list of formations, with add, clone and delete. Formations with issues get an error or warning icon.
- **Canvas and blocks tree**: the canvas draws a simulated deployment of the selected formation over a grid in meters, with the front at the top. The tree below shows each block under its parent. Selection is shared between both.
  - Drag blocks to move them. Their offsets snap to the grid step set above the canvas.
  - Shift+drag from a block to another to make the second one its parent.
  - Drag on empty space to select several blocks, and right-click for the block actions (add, add span over the selection, delete, delete with children).
  - **Units per Block** changes how many units the simulation assumes, so you can check how the formation stretches.
- **Inspector**: the properties of the selected formation, container or span, including AI purposes, unit category requirements, supported subcultures and factions, and each block's entity preferences.

Edits can be undone with **Ctrl+Z**. Deleting a block re-attaches its children to the closest surviving ancestor, keeping them in place. Issues like missing references or blocks depending on each other in a loop are reported in the [Diagnostics panel](../search/diagnostics.md).

> The simulated layout assumes a relative block's offset is a gap measured from its parent's edge, and that a zero offset centers it on the parent. This hasn't been confirmed in-game yet, so the canvas may not match the game exactly.

If you need the raw data, enable **Use Debug View for Group Formations** in the settings to open these files as JSON instead.

## Animation file formats summary

For convenience, here's where each animation-shaped format lives in the manual:

| Format                    | Editor chapter                         |
|---------------------------|----------------------------------------|
| AnimPack                  | [AnimPack](./animpack.md)              |
| AnimsTable                | [Animations](./animations.md)          |
| AnimFragmentBattle        | [Animations](./animations.md)          |
| MatchedCombat             | [Animations](./animations.md)          |
| `.anim` raw stream        | No UI viewer today                     |
