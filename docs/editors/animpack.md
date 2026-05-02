# AnimPack

An **AnimPack** (`.animpack`) is a container file CA uses to bundle animation-related game data — animation tables (`AnimsTable`), animation fragments (`AnimFragmentBattle`), matched-combat tables, and the raw `.bin` animation files referenced by them — into a single archive the engine can load efficiently. From a modder's perspective it's a "Pack inside a Pack".

<!-- IMAGE: AnimPack editor showing the open-Pack tree on the left and the AnimPack's internal tree on the right, with the instructions banner across the top. -->

## Layout

The editor opens as a single tab with two side-by-side panels and an instructions banner across the top.

- **Left panel** — a live view of the main contents tree of every open Pack. It mirrors what you see in the dockable Pack tree, including all open Packs simultaneously.
- **Right panel** — the contents of the AnimPack itself. Read-only display: you can browse and filter, but you can't open files into editors from here.

Each panel has its own filter line edit, case-sensitive toggle, auto-expand-matches toggle, and Expand-all / Collapse-all shortcuts.

## What you can do

The editor exposes exactly three operation:

- **Copy a file into the AnimPack** — double-click a file in the **left** panel. The file is copied from the open Pack into the AnimPack.
- **Copy a file out of the AnimPack** — double-click a file in the **right** panel. The file is copied out of the AnimPack into the open Pack at the same path.
- **Delete a file from the AnimPack** — select it on the right panel and press `Del` (the action is shortcut-bound; there is no visible context menu).

If the open AnimPack is from vanilla (`GameFiles`) or a parent mod (`ParentFiles`) rather than your own Pack, the copy-in action is silently disabled — you can only modify AnimPacks you own. Copy-out from a vanilla AnimPack into your own Pack is allowed.

## Which Pack the operations target (multiple Packs open)

Every operation resolves the target Pack from the **left panel's own selection** inside the AnimPack tab (or, if nothing is selected there, the first editable Pack in the panel). The dock's selection is irrelevant here.

- **Copy in**: source files come from the Pack of the file you double-clicked on the left panel, and the AnimPack inside that same Pack is what gets modified.
- **Copy out**: the file is written into the Pack currently selected (or first) on the left panel of the AnimPack tab. If that Pack doesn't have an AnimPack at the same path, the operation errors.
- **Delete**: operates on the AnimPack inside the Pack currently selected (or first) on the left panel.

So to switch which Pack you're working against, click a node belonging to that Pack inside the AnimPack tab's left panel — not in the dockable contents tree.

## Caveats

- Renaming files inside an AnimPack is not supported in the editor — you'd have to copy out, rename in the parent Pack, copy back in, and delete the original.
