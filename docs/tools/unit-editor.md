# Unit Editor

The Unit Editor is a higher-level alternative to editing units row-by-row across `main_units_tables`, `land_units_tables`, `unit_variants_tables` and the loc strings that name them. It gives you a single pane that pulls in the relevant rows from each table and presents the unit as a coherent thing.

> **Disabled by default.** The Unit Editor lives behind the **Enable Unit Editor** debug toggle in **PackFile → Settings → Debug**. Turn it on first; the menu entry is greyed out otherwise. Treat it as a power-user / experimental tool.
>
> **Warhammer 2 and Warhammer 3 only.** The tool's game-config map only covers WH2 and WH3 today. The menu entry is enabled for any game once the toggle is on, but trying to open it on an unsupported game will fail.

Open from **Tools → Unit Editor** with a Pack open (and the debug toggle enabled).

<!-- IMAGE: Unit Editor showing the unit list on the left, the unit's combined fields (main_units, land_units, variant references) in the middle, and a variant editor on the right. -->

## What it covers

For each unit, the editor surfaces:

- The `main_units_tables` row.
- The `land_units_tables` row.
- The unit's variant mesh references (`unit_variants_tables` etc., via the bundled variant editor).
- Loc strings (display name, description, short description, and — for WH2 — `strengths_weaknesses_texts`).

Editing any field writes back into the right DB row when you save. Cloning a unit propagates the related rows so they stay consistent.

> **Not currently covered**, despite what older docs claimed: `unit_stats_*_tables`, cost / upkeep / recruitment-time tables, and the unit caps tables. If you need to tweak those, edit the tables directly in the [DB editor](../editors/db.md).

## Per-game scope

Like Faction Painter, the Unit Editor reads game-specific table and column names from its game-config map. Only Warhammer 2 and Warhammer 3 are wired up today.

## Limitations

- **Add / delete are limited.** The primary "add" mechanism is **Clone** an existing unit; there's no "delete unit" action.
- For wholesale unit-creation workflows (full kit replacement, animations, sound, custom stats), the conventional table-by-table approach is still required.

## Tip

Cloning an existing unit and tweaking it is the most reliable way to add a new unit through this editor. Start from a vanilla unit close to what you want and go from there.
