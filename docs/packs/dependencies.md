# Dependencies

"Dependencies" in RPFM means two related but distinct things:

1. The **vanilla game data** plus any **parent mods** you tell RPFM to consider when looking up references, running diagnostics, detecting ITM rows, and so on. This is the **dependency cache**.
2. The **list of parent Packs** declared inside *your* Pack. This is the **dependency manager**.

This chapter covers both.

## The dependency cache

Most "smart" features in RPFM (reference lookups, table validation, ITM detection, datacore handling, the diagnostics panel, the optimiser, global search across vanilla, MyMod install) need to know what the vanilla game contains. RPFM keeps a cached snapshot of that data per-game.

### Generating the cache

**Game Selected → Generate Dependencies Cache** builds (or refreshes) the cache for the currently active game. The generation can take a few minutes — RPFM is reading every vanilla Pack of the game. Re-run it whenever:

- The game gets a content patch.
- The schemas got updated.
- The cache appears empty/missing files in the Dependencies panel.

The cache is stored under RPFM's config directory and is per-game.

### Assembly Kit data

For games with an Assembly Kit, the cache build also picks up AK-only tables (units, buildings, technologies, etc. that aren't shipped inside Packs) and stores them alongside the vanilla data. This happens automatically as part of **Generate Dependencies Cache** — there is no separate "Generate AK Database" command.

For this to work, RPFM needs to know where the Assembly Kit is installed. Set the path in **Preferences → Paths → Assembly Kit** for each game you have one for. If the path isn't configured, the cache is still built but RPFM warns that diagnostics may produce false positives because some reference tables won't be available.

### The Dependencies panel

The **Dependencies** dock (toggle from **View → Toggle Dependencies Window**) lets you browse:

- **Game Files** — vanilla Packs, indexed.
- **Parent Files** — files contributed by parent mods you've declared.
- **Assembly Kit Files** — Assembly-Kit-only tables and locs.

You can open any vanilla / parent / AK file in a read-only editor — useful for checking how Creative Assembly does things, or comparing your overrides against their source.

<!-- IMAGE: Dependencies panel expanded with Game Files, Parent Files and Assembly Kit Files visible, and a vanilla DB table opened in a read-only editor. -->

## The in-Pack dependency manager

A Pack can declare other Packs as **parents**. The game itself ignores this list, but RPFM uses it to identify the packs to load as part of the dependencies when opening the pack.

To edit a Pack's declared parents, right-click the Pack in the tree → **Open ▸ Open Dependency Manager**. It opens as a tab in the main view.

<!-- IMAGE: Dependency Manager tab showing a two-column table with the "hard" flag and the parent Pack name. -->

The manager is a small two-column table:

- **Load before ingame?** — checkbox. Forces the game to load the pack as dependency if present. Alters the load order and is not recommended to check it.
- **Pack** — the parent Pack file name (e.g. `parent_mod.pack`).

Order can matter for some games — when in doubt, list parents in load order.

> **Setting parents matters for diagnostics.** Without them declared, RPFM treats parent-mod content as missing and may report false positives in the diagnostics panel. If you're modding on top of someone else's mod, declare it.

## Putting it together

A typical workflow when you start a new mod project:

1. Make sure the dependencies cache is built for your game (and that the Assembly Kit path is set in Preferences if the game has one).
2. In your new Pack, open the Dependency Manager and declare any parent mods you depend on.
3. From here on, [Diagnostics](../search/diagnostics.md), [References](../search/references.md), DB editor lookups and the [Optimiser](../tutorials/optimising-a-mod.md) will all work correctly against your specific load order.
