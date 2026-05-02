# Faction Painter

The Faction Painter lets you edit faction colour schemes — primary / secondary / tertiary banner colours, uniform colours, and the related per-faction display knobs — without manually grinding through `factions_tables` and the colour columns.

Open from **Tools → Faction Painter** with a Pack open.

<!-- IMAGE: Faction Painter window with a faction list on the left, the colour swatches and pickers in the middle, and the live preview (where supported) on the right. -->

## Workflow

1. Open the Pack you want to add faction colour overrides to.
2. Open Faction Painter.
3. Pick the faction in the list.
4. Adjust the colours (or hit **Restore Vanilla Values** to baseline first).
5. **OK** — RPFM writes the new values into the appropriate DB row in your Pack, creating it as an override if it didn't exist already.

Edits are reflected immediately in any open DB editor for the affected tables (the tool tracks which paths it touched and reloads any matching open views after save).

## Per-game caveats

Different games store faction colours in different tables. Faction Painter resolves the right table and column names per game from its game-config map, so the picker UI stays the same across games even when the underlying schema doesn't.
