# Datacores (twad_key_deletes)

A short tour of an old, painful pattern in Total War modding — and how Warhammer 3 (patch 6.3+) and RPFM let you stop using it.

## What's a datacore?

To explain what a datacore is, first we need to explain how table rows are loaded by the engine. Imagine the following table layout:

- `db/whatever_tables/@table` — your high-priority overrides.
- `db/whatever_tables/ztable` — your low-priority overrides.
- `db/whatever_tables/data__` — the vanilla table (filename varies per game).

When loading in-game, the three are merged. When two rows of the merged table have the same key, **only the first one wins**. So:

- `@table` contains new rows and rows intended to overwrite vanilla. To change a vanilla unit's stats, you put the modified row here.
- `ztable` contains new rows you want overridden by other mods if they happen to clash (vanilla fixes, etc.).
- `data__` is the vanilla data.

Notice the gap: there's no clean way to **remove** a vanilla row. To delete a building effect, the historical workaround was to import the whole vanilla table into your mod with the offending row removed, then ship that. **That's a datacore.**

## Why datacores are bad

Imagine you have a fancy effects table and you want to remove one of the effects in a submod. You import the table into your submod, edit it, and it works. Until... the parent mod is updated to add new effects. Now your submodded table doesn't have those new effects, and either you update your table with them or your submod silently strips them.

And what if another submod tries to remove a different effect? Highlander situation: **there can only be one**.

In summary: datacoring is sometimes useful, but it reduces compatibility, makes the mod way bigger than it needs to be, and increases maintenance burden. Avoid unless absolutely needed.

For Warhammer 3 from patch 6.3 onwards, you don't have to.

## Datacores BEGONE: `twad_key_deletes_tables`

Patch 6.3 introduced a new table type, `twad_key_deletes_tables`, with a simple shape:

| Column      | Description |
|-------------|-------------|
| `table_name` | The table to delete a row from. |
| `key`       | The row's primary key (or composite key). |

The engine reads this table at runtime and deletes those rows from the named tables — replacing the need for datacores entirely.

There are three ways to use it.

### 1. Migrate existing datacores

If your mod already contains datacored tables and you want to convert them:

1. Right-click your Pack root → **Optimize PackFile** (or **Save Pack For Release** if you also want the optimiser to save the result).
2. In the optimiser dialog, tick **Import datacores into `twad_key_deletes`** under the Table section.
3. **Accept**.

RPFM analyses your datacored tables, finds the rows that are deleted relative to vanilla / parent, and adds them as entries to a new `twad_key_deletes` table. After running this, verify the new table contains the right keys, then delete the datacores.

> **Rename the generated table.** RPFM will overwrite it if you re-run the optimiser. Pick a stable name like `mymod_deletes`.

<!-- IMAGE: Optimizer dialog with the "Import datacores into twad_key_deletes" checkbox highlighted. -->

### 2. Add deletes from the source table

The clean ergonomic flow when you're removing entries you can see in another open table:

1. Right-click your Pack → **Create ▸ New DB**, type `twad_key_deletes_tables` as the table name, and create it once. Don't edit it manually.
2. Open the source table (the one with the rows you want gone).
3. Select the rows.
4. Right-click → **Add Selection to Key Deletes** → pick your `twad_key_deletes` table.

RPFM adds the right keys to the deletes table for you. No manual key copying — and especially no risk of mistyping a multi-key composite, which is the easiest way to add an entry that silently does nothing. The submenu is only enabled when the active game is Warhammer 3 and the destination table exists in the open Pack.

<!-- IMAGE: Right-click on a DB row showing the "Add Selection to Key Deletes" submenu. -->

### 3. Edit the table directly (last resort)

If neither of the above fits — for example, you want to delete a row that doesn't exist in any open table you can right-click — you can edit `twad_key_deletes` like any other DB table. This is the highest risk path: bad keys silently do nothing in-game and RPFM can't yet tell when you've fat-fingered a key. Use sparingly.

## Caveats

A few things to keep in mind:

- **Removing a referenced row crashes the game.** If you delete a `land_units` row that's still referenced by `units_to_groupings`, the game will crash on load — same as if you'd deleted the row in a datacore. Worse, the engine's error dialog tends to point at the wrong Pack, and RPFM's diagnostics don't yet validate `twad_key_deletes` references. If a vanilla row is referenced from elsewhere, plan accordingly.
- **Load order matters.** If your high-priority mod removes a row and a lower-priority mod re-adds it (or vice versa), the engine will reflect the load order — the deletion gets undone.

These aren't reasons to avoid `twad_key_deletes`; they're reasons to think before removing rows. Compared to datacores, it's still a massive ergonomic win.
