# Optimising a mod

Before you ship a release, optimise. RPFM's optimiser strips dead weight from a Pack — duplicated rows, rows identical to vanilla, modding-tool byproducts, unused portrait variants — and the result is smaller, more compatible, and less likely to break neighbouring mods.

## Why bother

- **Size.** Removing dead rows shrinks the Pack — sometimes dramatically.
- **Compatibility.** Duplicated and identical-to-vanilla rows tend to be left forgotten in mod packs (even when [Diagnostics](../search/diagnostics.md) warns about them) and end up causing compatibility issues when another mod tries to modify them. Your mod accidentally overwrites changes for no reason.
- **Multi-language support.** Identical-to-vanilla loc entries will overwrite the player's localisation files if the player is not on English. So even though you're not changing those lines, the optimiser-untouched Pack would silently turn them to English for non-English players. Bad.

## How

Two right-click actions on the Pack root:

- **Optimize PackFile** — opens the optimiser dialog, runs the steps you've ticked, applies the result to the in-memory Pack but doesn't save. Lets you review the result before committing.
- **Save Pack For Release** — opens the same optimiser dialog, runs it, *and* saves the result to disk. The shipping shortcut.

Both use the same `OptimizerOptions` under the hood. The dialog is where you fine-tune which steps run; your tick choices are persisted as defaults for the next invocation. There is no separate "Optimizer" pane in **PackFile → Settings** — the dialog itself is the configuration surface.

## When

Right before you publish a release / update. Not earlier — the optimiser strips data, and that data is occasionally what you're using as a reminder of intent during development.

## What it actually does

The full step list lives in [`rpfm_extensions::optimizer`](../../api/rpfm_extensions/optimizer/index.html), and the dialog groups them into four sections:

- **Pack** — Remove ITM (identical-to-master) files, Apply compression.
- **Table** (DB / Loc) — Import datacores into `twad_key_deletes` (Warhammer 3+, see [Datacores](./datacores.md)), Optimize datacored tables, Remove duplicated entries, Remove ITM rows, Remove ITNR (identical to new row) rows, Remove empty files.
- **Text** — Remove unused `.xml` files in `map/` folders, remove unused `.xml` files in the prefab folder, Remove `.agf` files, Remove `.model_statistics` files (the last two are BOB exporter byproducts).
- **Portrait Settings** — Remove unused art sets, Remove unused variants, Remove empty masks, Remove empty files.

Each step can be ticked or un-ticked from the dialog. The defaults are conservative — destructive steps like "Optimize datacored tables" and "Remove unused art sets / variants" are off out of the box.
