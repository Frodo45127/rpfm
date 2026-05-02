# Supported games

RPFM supports every Total War since *Empire*, with depth that varies depending on how interesting the game is to mod and how much the format has been reverse-engineered.

| Game                                | Game key             | Editing | Notes |
|-------------------------------------|----------------------|---------|-------|
| Total War: Pharaoh – Dynasties      | `pharaoh_dynasties`  | Full    | |
| Total War: Pharaoh                  | `pharaoh`            | Full    | |
| Total War: Warhammer 3              | `warhammer_3`        | Full    | First game with `twad_key_deletes` (patch 6.3+). |
| Total War Saga: Troy                | `troy`               | Full    | |
| Total War: Three Kingdoms           | `three_kingdoms`     | Full    | |
| Total War: Warhammer 2              | `warhammer_2`        | Full    | |
| Total War: Warhammer                | `warhammer`          | Full    | |
| Total War Saga: Thrones of Britannia| `thrones_of_britannia` | Full  | |
| Total War: Attila                   | `attila`             | Full    | |
| Total War: Rome 2                   | `rome_2`             | Full    | |
| Total War: Shogun 2                 | `shogun_2`           | Full    | |
| Total War: Napoleon                 | `napoleon`           | Full    | No Assembly Kit ever shipped — RPFM uses archived AK definitions. |
| Total War: Empire                   | `empire`             | Full    | No Assembly Kit ever shipped — RPFM uses archived AK definitions. |
| Total War: Arena                    | `arena`              | Read-only | Discontinued; `supports_editing` is `false`, so RPFM can browse Arena Packs but won't save them. |

## Game-specific quirks

A few rough edges worth knowing about:

- **Compression of DB tables** is only safe on Warhammer 3 and newer; older games crash on compressed table data, so the compression code skips DB tables for them. Pharaoh / Pharaoh Dynasties / Three Kingdoms / Troy / Warhammer 2 still support `Lzma1` for non-table files. Warhammer 3 adds `Lz4` and `Zstd` on top of `Lzma1`. Anything older than Warhammer 1 supports no compression at all.
- **`twad_key_deletes`** is Warhammer 3 (6.3+) only. See [Datacores](../tutorials/datacores.md).
- **Animation ID lookups** for Three Kingdoms, Troy, Warhammer 2 and Warhammer 3 use the per-game `anim_ids_*.csv` shipped with the [schemas repo](./schemas.md).
- **Group formations** were rebuilt for Warhammer 3 and adopt different on-disk formats per game; RPFM keeps each variant straight.
- **MatchedCombat** has different decode paths for Three Kingdoms vs Warhammer 3 vs the rest.

## What "full" support means

For "Full" games, RPFM can:

- Open and save Packs of every type the engine accepts.
- Decode and re-encode every commonly-edited file format (DB, Loc, Text, AnimPack, Animations, Portrait Settings, Atlas, Group Formations).
- Run diagnostics, global search, references, dependency analysis.
- Drive the optimiser, the translator, and (where applicable) startpos build, map packing, and SiegeAI patching.

Some less-common formats (BMD, ESF, RigidModel) are read-only or partially supported across all games — see [Editors overview](../editors/overview.md) for the per-format support level.

## Vanilla data lookups

For every game with a configured install path, RPFM builds a [dependency cache](../packs/dependencies.md) so that vanilla data is available for reference lookups, ITM detection, the optimiser, and more. The cache is per-game and needs regenerating when the game gets a content patch.
