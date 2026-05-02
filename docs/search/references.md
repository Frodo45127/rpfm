# References panel

The References panel answers the question "where else is this referenced?" for any DB cell. It's the modder's "find all usages" — invaluable before deleting, renaming, or changing a row whose key something else depends on.

Toggle the panel from **View → Toggle References Window**.

<!-- IMAGE: References panel listing every reference to a unit's key across DB tables and locs, with the source row visible for each. -->

## Triggering a reference search

In any DB editor, right-click a cell whose value is a key (typically a primary key or a foreign-key value) and choose **Find References**. The References panel populates with every other place that key appears — across the open Pack, parent mods, vanilla, and (where relevant) the Assembly Kit.

## Reading the results

Each result is a row in a small table with these columns:

- **Data Source** — `PackFile`, `ParentFiles`, `GameFiles`, or `AssKitFiles`.
- **Path** — the in-Pack path of the file containing the reference.
- **Column Name** — the column that's referencing the key (DB-only; blank for loc/filename refs).
- **Row Number** — the row containing the reference.

(There's also a Column Number column, hidden by default — you can re-show it from the table header's right-click menu if you need it.)

Click a result to jump to it.

## Common workflows

- **Before deleting a row.** Run a reference search on the row's key. If anything in your Pack references it, you'll need to clean those up first. References from vanilla / parent mods mean the row probably shouldn't be deleted at all.
- **Before renaming a row.** Same as above. If the references are all in your own Pack, [Cascade Edition](../editors/db.md) (right-click → Cascade Edition in the DB editor) can do the rename across them in one shot.
- **Auditing a new feature.** After adding a new unit, run a reference search on its key to confirm it's wired up everywhere it should be (cost table, recruitment table, faction availability, etc.).
- **Understanding a vanilla mechanic.** Open a vanilla DB table from the [Dependencies panel](../packs/dependencies.md), reference-search a key that interests you, and follow the trail.

## Limitations

References are computed from declared schema relationships — RPFM knows that column X in table A references column Y in table B because the schema says so. Hard-coded references in script files (Lua) or text-formatted UI files won't show up unless the schema knows about them.

For text-only references (e.g. a Lua script that looks up a unit by string), use [Global Search](./global-search.md) instead.
